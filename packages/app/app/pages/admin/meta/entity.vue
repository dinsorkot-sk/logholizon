<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

import { h, resolveComponent } from 'vue'
import type { TableColumn } from '@nuxt/ui'

const UButton = resolveComponent('UButton')
const UBadge = resolveComponent('UBadge')

type Entity = { id: string; name: string; label: string; module?: string | null }
type FieldOption = { id: string; value: string; label: string }
type Field = { id: string; name: string; type: string; required: boolean; is_status: boolean; position: number; ref_entity?: string | null; computed_expr?: string | null; options: FieldOption[] }
type EntityDetail = Entity & { fields: Field[] }
type WorkflowState = { id: string; name: string; label: string; position: number }
type WorkflowTransition = { id: string; action: string; from_state: string; to_state: string }
type WorkflowDefinition = { states: WorkflowState[]; transitions: WorkflowTransition[] }
type EntityPermission = { role: string; can_view: boolean; can_edit: boolean }
type FieldPermission = { field_id: string; role: string; can_view: boolean; can_edit: boolean }
type EntityView = { id: string; entity_id: string; name: string; config: Record<string, unknown>; created_at: string }
type FormLayoutSection = { id: string; label: string; fields: string[] }
type FormLayout = { entity_id: string; config: { sections?: FormLayoutSection[] } }
type NotificationRule = { id: string; entity_id: string; trigger: string; target_url: string; active: boolean; created_at: string }

const toast = useToast()
const { data: entities, status, refresh } = await useFetch<Entity[]>('/api/meta/entities')

const search = ref('')
const selectedId = ref('')
const detailUrl = computed(() => selectedId.value ? `/api/meta/entities/${encodeURIComponent(selectedId.value)}` : '')
const { data: detail, status: detailStatus, error: detailError, refresh: refreshDetail } = await useFetch<EntityDetail>(detailUrl, { immediate: false })
const workflowUrl = computed(() => selectedId.value ? `/api/meta/entities/${encodeURIComponent(selectedId.value)}/workflow` : '')
const { data: workflow, status: workflowStatus, error: workflowError, refresh: refreshWorkflow } = await useFetch<WorkflowDefinition>(workflowUrl, { immediate: false })
const permissionsUrl = computed(() => selectedId.value ? `/api/meta/entities/${encodeURIComponent(selectedId.value)}/permissions` : '')
const { data: permissions, status: permissionsStatus, error: permissionsError, refresh: refreshPermissions } = await useFetch<EntityPermission[]>(permissionsUrl, { immediate: false })
const fieldPermissionsUrl = computed(() => selectedId.value ? `/api/meta/entities/${encodeURIComponent(selectedId.value)}/field-permissions` : '')
const { data: fieldPermissions, status: fieldPermissionsStatus, error: fieldPermissionsError, refresh: refreshFieldPermissions } = await useFetch<FieldPermission[]>(fieldPermissionsUrl, { immediate: false })
const viewsUrl = computed(() => selectedId.value ? `/api/meta/entities/${encodeURIComponent(selectedId.value)}/views` : '')
const { data: views, status: viewsStatus, error: viewsError, refresh: refreshViews } = await useFetch<EntityView[]>(viewsUrl, { immediate: false })
const formLayoutUrl = computed(() => selectedId.value ? `/api/meta/entities/${encodeURIComponent(selectedId.value)}/form-layout` : '')
const { data: formLayout, status: formLayoutStatus, error: formLayoutError, refresh: refreshFormLayout } = await useFetch<FormLayout>(formLayoutUrl, { immediate: false })
const notifyRulesUrl = computed(() => selectedId.value ? `/api/meta/entities/${encodeURIComponent(selectedId.value)}/notification-rules` : '')
const { data: notifyRules, status: notifyRulesStatus, error: notifyRulesError, refresh: refreshNotifyRules } = await useFetch<NotificationRule[]>(notifyRulesUrl, { immediate: false })

const filteredEntities = computed(() => {
  const q = search.value.trim().toLowerCase()
  if (!q) return entities.value || []
  return (entities.value || []).filter(e =>
    e.id.toLowerCase().includes(q) || e.label.toLowerCase().includes(q)
  )
})

const tabItems = [
  { label: 'Fields', icon: 'i-lucide-table', slot: 'fields' },
  { label: 'Workflow', icon: 'i-lucide-git-branch', slot: 'workflow' },
  { label: 'Permissions', icon: 'i-lucide-shield', slot: 'permissions' },
  { label: 'Views', icon: 'i-lucide-eye', slot: 'views' },
  { label: 'Form Layout', icon: 'i-lucide-layout-dashboard', slot: 'form-layout' },
  { label: 'Notifications', icon: 'i-lucide-bell', slot: 'notifications' }
]

const typeItems = [
  { label: 'Text', value: 'text', icon: 'i-lucide-type' },
  { label: 'Number', value: 'number', icon: 'i-lucide-hash' },
  { label: 'Date', value: 'date', icon: 'i-lucide-calendar' },
  { label: 'Select', value: 'select', icon: 'i-lucide-chevrons-up-down' },
  { label: 'Checkbox', value: 'checkbox', icon: 'i-lucide-square-check' },
  { label: 'Textarea', value: 'textarea', icon: 'i-lucide-align-left' },
  { label: 'Currency', value: 'currency', icon: 'i-lucide-dollar-sign' },
  { label: 'Reference', value: 'reference', icon: 'i-lucide-link' },
  { label: 'Computed', value: 'computed', icon: 'i-lucide-function-square' }
]

function typeBadgeColor(type: string) {
  switch (type) {
    case 'text': return 'info'
    case 'select': return 'primary'
    case 'number': return 'warning'
    case 'checkbox': return 'success'
    case 'currency': return 'success'
    case 'reference': return 'info'
    case 'computed': return 'neutral'
    default: return 'neutral'
  }
}

function selectEntity(id: string) {
  selectedId.value = id
  refreshDetail()
  refreshWorkflow()
  refreshPermissions()
  refreshFieldPermissions()
  refreshViews()
  refreshFormLayout()
  resetLayoutEditor()
  refreshNotifyRules()
}

// --- Permissions ---
const savingPermissions = ref(false)

async function togglePermission(permission: EntityPermission, key: 'can_view' | 'can_edit') {
  if (!selectedId.value || !permissions.value) return
  const next = permissions.value.map(p =>
    p.role === permission.role ? { ...p, [key]: !p[key] } : { ...p }
  )
  savingPermissions.value = true
  try {
    await $fetch(`/api/meta/entities/${encodeURIComponent(selectedId.value)}/permissions`, {
      method: 'PUT',
      body: { permissions: next }
    })
    await refreshPermissions()
    toast.add({ title: 'Permissions updated', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    toast.add({
      title: 'Unable to update permissions',
      description: e?.data?.message || e?.statusMessage || 'Update failed',
      color: 'error',
      icon: 'i-lucide-alert-circle'
    })
  } finally {
    savingPermissions.value = false
  }
}

const savingFieldPermissions = ref(false)

function fieldPermissionFor(fieldId: string) {
  return (fieldPermissions.value || []).find(p => p.field_id === fieldId && p.role === 'user')
}

async function toggleFieldPermission(fieldId: string, key: 'can_view' | 'can_edit') {
  if (!selectedId.value || !fieldPermissions.value) return
  const current = fieldPermissionFor(fieldId)
  const next = { field_id: fieldId, role: 'user', can_view: current?.can_view ?? true, can_edit: current?.can_edit ?? true }
  next[key] = !next[key]
  if (!next.can_view) next.can_edit = false
  savingFieldPermissions.value = true
  try {
    await $fetch(`/api/meta/entities/${encodeURIComponent(selectedId.value)}/field-permissions`, {
      method: 'PUT',
      body: { permissions: [next] }
    })
    await refreshFieldPermissions()
    toast.add({ title: 'Field permissions updated', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    toast.add({
      title: 'Unable to update field permissions',
      description: e?.data?.message || e?.statusMessage || 'Update failed',
      color: 'error',
      icon: 'i-lucide-alert-circle'
    })
  } finally {
    savingFieldPermissions.value = false
  }
}

// --- Views ---
const viewOpen = ref(false)
const viewForm = reactive({ name: '' })
const viewError = ref('')
const savingView = ref(false)
const deleteViewOpen = ref(false)
const viewToDelete = ref<EntityView | null>(null)
const deletingView = ref(false)

function openAddView() {
  viewForm.name = ''
  viewError.value = ''
  viewOpen.value = true
}

async function saveView() {
  if (!selectedId.value) return
  viewError.value = ''
  if (!viewForm.name.trim()) {
    viewError.value = 'name is required'
    return
  }
  savingView.value = true
  try {
    await $fetch(`/api/meta/entities/${encodeURIComponent(selectedId.value)}/views`, {
      method: 'POST',
      body: { name: viewForm.name, config: {} }
    })
    viewOpen.value = false
    await refreshViews()
    toast.add({ title: 'View created', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    viewError.value = e?.data?.message || e?.statusMessage || 'Failed to create view'
  } finally {
    savingView.value = false
  }
}

// --- Form Layout ---
const layoutSections = ref<FormLayoutSection[]>([])
const layoutDirty = ref(false)
const savingLayout = ref(false)
const layoutError = ref('')
const sectionForm = reactive({ label: '' })
const sectionOpen = ref(false)

function layoutFieldIds(sections: FormLayoutSection[]) {
  return new Set(sections.flatMap(s => s.fields))
}

function resetLayoutEditor() {
  const sections = formLayout.value?.config?.sections
  layoutSections.value = Array.isArray(sections)
    ? sections.map(s => ({ id: String(s.id), label: String(s.label || s.id), fields: [...(s.fields || [])] }))
    : []
  layoutDirty.value = false
  layoutError.value = ''
}

watch(formLayout, () => {
  if (!layoutDirty.value) resetLayoutEditor()
})

const unassignedFields = computed(() => {
  const assigned = layoutFieldIds(layoutSections.value)
  return (detail.value?.fields || []).filter(f => !f.is_status && !assigned.has(f.id))
})

const layoutPreview = computed(() => {
  const byId = new Map((detail.value?.fields || []).map(f => [f.id, f]))
  const sections = layoutSections.value
    .map(s => ({ id: s.id, label: s.label, fields: s.fields.map(id => byId.get(id)).filter(Boolean) as Field[] }))
    .filter(s => s.fields.length > 0)
  const assigned = new Set(sections.flatMap(s => s.fields.map(f => f.id)))
  const other = (detail.value?.fields || []).filter(f => !f.is_status && !assigned.has(f.id))
  if (other.length) sections.push({ id: 'other', label: 'Other', fields: other })
  return sections
})

function markLayoutDirty() {
  layoutDirty.value = true
}

function openAddSection() {
  sectionForm.label = ''
  layoutError.value = ''
  sectionOpen.value = true
}

function saveSection() {
  layoutError.value = ''
  const label = sectionForm.label.trim()
  if (!label) {
    layoutError.value = 'section label is required'
    return
  }
  const id = label.toLowerCase().replace(/[^a-z0-9]+/g, '_').replace(/^_|_$/g, '') || `section_${layoutSections.value.length + 1}`
  if (layoutSections.value.some(s => s.id === id)) {
    layoutError.value = `section already exists: ${id}`
    return
  }
  layoutSections.value.push({ id, label, fields: [] })
  sectionOpen.value = false
  markLayoutDirty()
}

function removeSection(index: number) {
  layoutSections.value.splice(index, 1)
  markLayoutDirty()
}

function moveSection(index: number, dir: -1 | 1) {
  const next = index + dir
  if (next < 0 || next >= layoutSections.value.length) return
  const [section] = layoutSections.value.splice(index, 1)
  if (!section) return
  layoutSections.value.splice(next, 0, section)
  markLayoutDirty()
}

function moveField(sectionIndex: number, fieldIndex: number, dir: -1 | 1) {
  const fields = layoutSections.value[sectionIndex]?.fields
  if (!fields) return
  const next = fieldIndex + dir
  if (next < 0 || next >= fields.length) return
  const [field] = fields.splice(fieldIndex, 1)
  if (field === undefined) return
  fields.splice(next, 0, field)
  markLayoutDirty()
}

function moveFieldToSection(fromSection: number, fieldIndex: number, toSection: number) {
  if (toSection < 0 || toSection >= layoutSections.value.length || fromSection === toSection) return
  const from = layoutSections.value[fromSection]?.fields
  const to = layoutSections.value[toSection]?.fields
  if (!from || !to) return
  const [field] = from.splice(fieldIndex, 1)
  if (field === undefined) return
  to.push(field)
  markLayoutDirty()
}

function removeFieldFromLayout(sectionIndex: number, fieldIndex: number) {
  layoutSections.value[sectionIndex]?.fields.splice(fieldIndex, 1)
  markLayoutDirty()
}

function addFieldToSection(sectionIndex: number, fieldId: string) {
  if (!fieldId || layoutFieldIds(layoutSections.value).has(fieldId)) return
  layoutSections.value[sectionIndex]?.fields.push(fieldId)
  markLayoutDirty()
}

function fieldName(fieldId: string) {
  return (detail.value?.fields || []).find(f => f.id === fieldId)?.name || fieldId
}

async function saveLayout() {
  if (!selectedId.value) return
  layoutError.value = ''
  savingLayout.value = true
  try {
    await $fetch(`/api/meta/entities/${encodeURIComponent(selectedId.value)}/form-layout`, {
      method: 'PUT',
      body: { config: { sections: layoutSections.value } }
    })
    layoutDirty.value = false
    await refreshFormLayout()
    toast.add({ title: 'Form layout saved', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    layoutError.value = e?.data?.message || e?.statusMessage || 'Failed to save layout'
  } finally {
    savingLayout.value = false
  }
}

// --- Notifications ---
const ruleOpen = ref(false)
const ruleForm = reactive({ target_url: '', active: true })
const ruleError = ref('')
const savingRule = ref(false)
const deleteRuleOpen = ref(false)
const ruleToDelete = ref<NotificationRule | null>(null)
const deletingRule = ref(false)
const togglingRuleId = ref('')

function openAddRule() {
  ruleForm.target_url = ''
  ruleForm.active = true
  ruleError.value = ''
  ruleOpen.value = true
}

async function saveRule() {
  if (!selectedId.value) return
  ruleError.value = ''
  if (!ruleForm.target_url.trim()) {
    ruleError.value = 'target_url is required'
    return
  }
  savingRule.value = true
  try {
    await $fetch(`/api/meta/entities/${encodeURIComponent(selectedId.value)}/notification-rules`, {
      method: 'POST',
      body: { trigger: 'transition', target_url: ruleForm.target_url, active: ruleForm.active }
    })
    ruleOpen.value = false
    await refreshNotifyRules()
    toast.add({ title: 'Notification rule created', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    ruleError.value = e?.data?.message || e?.statusMessage || 'Failed to create rule'
  } finally {
    savingRule.value = false
  }
}

async function toggleRule(rule: NotificationRule) {
  togglingRuleId.value = rule.id
  try {
    await $fetch(`/api/meta/notification-rules/${encodeURIComponent(rule.id)}`, {
      method: 'PUT',
      body: { active: !rule.active }
    })
    await refreshNotifyRules()
    toast.add({ title: rule.active ? 'Rule disabled' : 'Rule enabled', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    toast.add({
      title: 'Unable to update rule',
      description: e?.data?.message || e?.statusMessage || 'Update failed',
      color: 'error',
      icon: 'i-lucide-alert-circle'
    })
  } finally {
    togglingRuleId.value = ''
  }
}

function confirmDeleteRule(rule: NotificationRule) {
  ruleToDelete.value = rule
  deleteRuleOpen.value = true
}

async function removeRule() {
  if (!ruleToDelete.value) return
  deletingRule.value = true
  try {
    await $fetch(`/api/meta/notification-rules/${encodeURIComponent(ruleToDelete.value.id)}`, { method: 'DELETE' })
    deleteRuleOpen.value = false
    ruleToDelete.value = null
    await refreshNotifyRules()
    toast.add({ title: 'Rule deleted', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    toast.add({
      title: 'Unable to delete rule',
      description: e?.data?.message || e?.statusMessage || 'Delete failed',
      color: 'error',
      icon: 'i-lucide-alert-circle'
    })
  } finally {
    deletingRule.value = false
  }
}

function confirmDeleteView(view: EntityView) {
  viewToDelete.value = view
  deleteViewOpen.value = true
}

async function removeView() {
  if (!viewToDelete.value) return
  deletingView.value = true
  try {
    await $fetch(`/api/meta/views/${encodeURIComponent(viewToDelete.value.id)}`, { method: 'DELETE' })
    deleteViewOpen.value = false
    viewToDelete.value = null
    await refreshViews()
    toast.add({ title: 'View deleted', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    toast.add({
      title: 'Unable to delete view',
      description: e?.data?.message || e?.statusMessage || 'Delete failed',
      color: 'error',
      icon: 'i-lucide-alert-circle'
    })
  } finally {
    deletingView.value = false
  }
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
const editForm = reactive({ name: '', label: '', module: '' })
const editing = ref(false)
const editError = ref('')

function openEditEntity() {
  if (!detail.value) return
  editForm.name = detail.value.name
  editForm.label = detail.value.label
  editForm.module = detail.value.module || ''
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
const fieldForm = reactive({ name: '', type: 'text', required: false, is_status: false, ref_entity: '', computed_expr: '' })
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
  fieldForm.is_status = false
  fieldForm.ref_entity = ''
  fieldForm.computed_expr = ''
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
  fieldForm.is_status = field.is_status
  fieldForm.ref_entity = field.ref_entity || ''
  fieldForm.computed_expr = field.computed_expr || ''
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
        body: { name: fieldForm.name, type: fieldForm.type, required: fieldForm.required, is_status: fieldForm.is_status, ref_entity: fieldForm.ref_entity || null, computed_expr: fieldForm.computed_expr || null }
      })
      fieldOpen.value = false
      await refreshDetail()
      toast.add({ title: 'Field updated', color: 'success', icon: 'i-lucide-check' })
    } else {
      const created = await $fetch<Field>(`/api/meta/entities/${encodeURIComponent(detail.value.id)}/fields`, {
        method: 'POST',
        body: { name: fieldForm.name, type: fieldForm.type, required: fieldForm.required, is_status: fieldForm.is_status, ref_entity: fieldForm.ref_entity || null, computed_expr: fieldForm.computed_expr || null }
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

// --- Workflow editor ---
const stateOpen = ref(false)
const editingState = ref<WorkflowState | null>(null)
const stateForm = reactive({ name: '', label: '' })
const stateError = ref('')
const savingState = ref(false)
const deleteStateOpen = ref(false)
const stateToDelete = ref<WorkflowState | null>(null)
const deletingState = ref(false)

const transitionOpen = ref(false)
const transitionForm = reactive({ from_state: '', to_state: '', action: '' })
const transitionError = ref('')
const savingTransition = ref(false)
const deleteTransitionOpen = ref(false)
const transitionToDelete = ref<WorkflowTransition | null>(null)
const deletingTransition = ref(false)

function openAddState() {
  editingState.value = null
  stateForm.name = ''
  stateForm.label = ''
  stateError.value = ''
  stateOpen.value = true
}

function openEditState(state: WorkflowState) {
  editingState.value = state
  stateForm.name = state.name
  stateForm.label = state.label
  stateError.value = ''
  stateOpen.value = true
}

async function saveState() {
  if (!selectedId.value) return
  stateError.value = ''
  if (!stateForm.name.trim() || !stateForm.label.trim()) {
    stateError.value = 'name and label are required'
    return
  }
  if (!/^[a-z][a-z0-9_]*$/.test(stateForm.name)) {
    stateError.value = 'name must be lowercase snake_case (e.g. open)'
    return
  }
  savingState.value = true
  try {
    if (editingState.value) {
      await $fetch(`/api/meta/workflow/states/${encodeURIComponent(editingState.value.id)}`, {
        method: 'PUT',
        body: { label: stateForm.label }
      })
      toast.add({ title: 'State updated', color: 'success', icon: 'i-lucide-check' })
    } else {
      await $fetch(`/api/meta/entities/${encodeURIComponent(selectedId.value)}/workflow/states`, {
        method: 'POST',
        body: { name: stateForm.name, label: stateForm.label }
      })
      toast.add({ title: 'State created', color: 'success', icon: 'i-lucide-check' })
    }
    stateOpen.value = false
    await refreshWorkflow()
  } catch (e: any) {
    stateError.value = e?.data?.message || e?.statusMessage || 'Failed to save state'
  } finally {
    savingState.value = false
  }
}

function confirmDeleteState(state: WorkflowState) {
  stateToDelete.value = state
  deleteStateOpen.value = true
}

async function removeState() {
  if (!stateToDelete.value) return
  deletingState.value = true
  try {
    await $fetch(`/api/meta/workflow/states/${encodeURIComponent(stateToDelete.value.id)}`, { method: 'DELETE' })
    deleteStateOpen.value = false
    stateToDelete.value = null
    await refreshWorkflow()
    toast.add({ title: 'State deleted', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    toast.add({
      title: 'Unable to delete state',
      description: e?.data?.message || e?.statusMessage || 'Delete failed',
      color: 'error',
      icon: 'i-lucide-alert-circle'
    })
  } finally {
    deletingState.value = false
  }
}

function openAddTransition() {
  transitionForm.from_state = ''
  transitionForm.to_state = ''
  transitionForm.action = ''
  transitionError.value = ''
  transitionOpen.value = true
}

async function saveTransition() {
  if (!selectedId.value) return
  transitionError.value = ''
  if (!transitionForm.from_state.trim() || !transitionForm.to_state.trim() || !transitionForm.action.trim()) {
    transitionError.value = 'from, to, and action are required'
    return
  }
  if (!/^[a-z][a-z0-9_]*$/.test(transitionForm.action)) {
    transitionError.value = 'action must be lowercase snake_case (e.g. submit)'
    return
  }
  if (transitionForm.from_state === transitionForm.to_state) {
    transitionError.value = 'from and to must differ'
    return
  }
  savingTransition.value = true
  try {
    await $fetch(`/api/meta/entities/${encodeURIComponent(selectedId.value)}/workflow/transitions`, {
      method: 'POST',
      body: { ...transitionForm }
    })
    transitionOpen.value = false
    await refreshWorkflow()
    toast.add({ title: 'Transition created', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    transitionError.value = e?.data?.message || e?.statusMessage || 'Failed to save transition'
  } finally {
    savingTransition.value = false
  }
}

function confirmDeleteTransition(transition: WorkflowTransition) {
  transitionToDelete.value = transition
  deleteTransitionOpen.value = true
}

async function removeTransition() {
  if (!transitionToDelete.value) return
  deletingTransition.value = true
  try {
    await $fetch(`/api/meta/workflow/transitions/${encodeURIComponent(transitionToDelete.value.id)}`, { method: 'DELETE' })
    deleteTransitionOpen.value = false
    transitionToDelete.value = null
    await refreshWorkflow()
    toast.add({ title: 'Transition deleted', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    toast.add({
      title: 'Unable to delete transition',
      description: e?.data?.message || e?.statusMessage || 'Delete failed',
      color: 'error',
      icon: 'i-lucide-alert-circle'
    })
  } finally {
    deletingTransition.value = false
  }
}

const stateColumns: TableColumn<WorkflowState>[] = [
  {
    accessorKey: 'name',
    header: 'State',
    cell: ({ row }) => h('span', { class: 'font-mono' }, row.original.name)
  },
  {
    accessorKey: 'label',
    header: 'Label'
  },
  {
    id: 'actions',
    header: () => h('span', { class: 'sr-only' }, 'Actions'),
    cell: ({ row }) => h('div', { class: 'flex justify-end gap-1' }, [
      h(UButton, { size: 'xs', variant: 'ghost', onClick: () => openEditState(row.original) }, () => 'Edit'),
      h(UButton, { size: 'xs', variant: 'ghost', color: 'error', onClick: () => confirmDeleteState(row.original) }, () => 'Delete')
    ])
  }
]

const transitionColumns: TableColumn<WorkflowTransition>[] = [
  {
    accessorKey: 'from_state',
    header: 'From',
    cell: ({ row }) => h('span', { class: 'font-mono' }, row.original.from_state)
  },
  {
    accessorKey: 'action',
    header: 'Action',
    cell: ({ row }) => h(UBadge, { variant: 'subtle' }, () => row.original.action)
  },
  {
    accessorKey: 'to_state',
    header: 'To',
    cell: ({ row }) => h('span', { class: 'font-mono' }, row.original.to_state)
  },
  {
    id: 'actions',
    header: () => h('span', { class: 'sr-only' }, 'Actions'),
    cell: ({ row }) => h(UButton, { size: 'xs', variant: 'ghost', color: 'error', onClick: () => confirmDeleteTransition(row.original) }, () => 'Delete')
  }
]

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
    accessorKey: 'is_status',
    header: 'Status',
    cell: ({ row }) => row.original.is_status
      ? h(UBadge, { color: 'primary', variant: 'subtle' }, () => 'Status field')
      : ''
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
            <template #workflow>
              <div v-if="workflowStatus === 'pending'" class="py-6">
                <USkeleton v-for="index in 3" :key="index" class="mb-3 h-8 w-full" />
              </div>
              <UAlert
                v-else-if="workflowStatus === 'error'"
                color="error"
                title="Cannot load workflow"
                :description="workflowError?.message || 'Check the Rust core connection.'"
              >
                <template #actions>
                  <UButton size="sm" variant="outline" @click="refreshWorkflow()">Retry</UButton>
                </template>
              </UAlert>
              <div v-else class="space-y-6 py-3">
                <div>
                  <div class="flex items-center justify-between pb-2">
                    <p class="text-sm text-muted">{{ (workflow?.states || []).length }} states</p>
                    <UButton size="sm" icon="i-lucide-plus" @click="openAddState">Add state</UButton>
                  </div>
                  <UTable :data="workflow?.states || []" :columns="stateColumns" :get-row-id="(row: WorkflowState) => row.id" class="w-full">
                    <template #empty>
                      <div class="py-8 text-center text-muted">No states yet. Add the first state (e.g. draft).</div>
                    </template>
                  </UTable>
                </div>
                <div>
                  <div class="flex items-center justify-between pb-2">
                    <p class="text-sm text-muted">{{ (workflow?.transitions || []).length }} transitions</p>
                    <UButton size="sm" icon="i-lucide-plus" @click="openAddTransition">Add transition</UButton>
                  </div>
                  <UTable :data="workflow?.transitions || []" :columns="transitionColumns" :get-row-id="(row: WorkflowTransition) => row.id" class="w-full">
                    <template #empty>
                      <div class="py-8 text-center text-muted">No transitions yet. Connect two states with an action.</div>
                    </template>
                  </UTable>
                </div>
              </div>
            </template>
            <template #permissions>
              <div v-if="permissionsStatus === 'pending'" class="py-6">
                <USkeleton v-for="index in 2" :key="index" class="mb-3 h-8 w-full" />
              </div>
              <UAlert
                v-else-if="permissionsStatus === 'error'"
                color="error"
                title="Cannot load permissions"
                :description="permissionsError?.message || 'Check the Rust core connection.'"
              >
                <template #actions>
                  <UButton size="sm" variant="outline" @click="refreshPermissions()">Retry</UButton>
                </template>
              </UAlert>
              <div v-else class="space-y-2 py-3">
                <h3 class="text-sm font-semibold">Entity access</h3>
                <div v-for="permission in permissions || []" :key="permission.role" class="flex items-center justify-between gap-4 rounded-lg border border-default px-4 py-3">
                  <div>
                    <p class="font-mono text-sm font-medium">{{ permission.role }}</p>
                    <p class="text-xs text-muted">{{ permission.role === 'admin' ? 'Full access' : 'Limited by toggles' }}</p>
                  </div>
                  <div class="flex items-center gap-4">
                    <UFormField label="View" :ui="{ label: 'text-xs' }">
                      <USwitch :model-value="permission.can_view" :disabled="savingPermissions" @update:model-value="togglePermission(permission, 'can_view')" />
                    </UFormField>
                    <UFormField label="Edit" :ui="{ label: 'text-xs' }">
                      <USwitch :model-value="permission.can_edit" :disabled="savingPermissions || !permission.can_view" @update:model-value="togglePermission(permission, 'can_edit')" />
                    </UFormField>
                  </div>
                </div>
                <h3 class="pt-4 text-sm font-semibold">Field access (user role — admin always has full access)</h3>
                <div v-if="fieldPermissionsStatus === 'pending'" class="py-4">
                  <USkeleton v-for="index in 2" :key="index" class="mb-2 h-8 w-full" />
                </div>
                <UAlert
                  v-else-if="fieldPermissionsStatus === 'error'"
                  color="error"
                  title="Cannot load field permissions"
                  :description="fieldPermissionsError?.message || 'Check the Rust core connection.'"
                >
                  <template #actions>
                    <UButton size="sm" variant="outline" @click="refreshFieldPermissions()">Retry</UButton>
                  </template>
                </UAlert>
                <div v-else-if="!(detail?.fields || []).length" class="py-4 text-sm text-muted">No fields yet.</div>
                <div v-else class="space-y-2">
                  <div v-for="field in detail?.fields || []" :key="field.id" class="flex items-center justify-between gap-4 rounded-lg border border-default px-4 py-2">
                    <div>
                      <p class="font-mono text-sm font-medium">{{ field.name }}</p>
                      <p class="text-xs text-muted">{{ field.type }}{{ field.required ? ' · required' : '' }}{{ field.is_status ? ' · status' : '' }}</p>
                    </div>
                    <div class="flex items-center gap-4">
                      <UFormField label="View" :ui="{ label: 'text-xs' }">
                        <USwitch :model-value="fieldPermissionFor(field.id)?.can_view ?? true" :disabled="savingFieldPermissions" @update:model-value="toggleFieldPermission(field.id, 'can_view')" />
                      </UFormField>
                      <UFormField label="Edit" :ui="{ label: 'text-xs' }">
                        <USwitch :model-value="fieldPermissionFor(field.id)?.can_edit ?? true" :disabled="savingFieldPermissions || !(fieldPermissionFor(field.id)?.can_view ?? true)" @update:model-value="toggleFieldPermission(field.id, 'can_edit')" />
                      </UFormField>
                    </div>
                  </div>
                </div>
              </div>
            </template>
            <template #views>
              <div v-if="viewsStatus === 'pending'" class="py-6">
                <USkeleton v-for="index in 2" :key="index" class="mb-3 h-8 w-full" />
              </div>
              <UAlert
                v-else-if="viewsStatus === 'error'"
                color="error"
                title="Cannot load views"
                :description="viewsError?.message || 'Check the Rust core connection.'"
              >
                <template #actions>
                  <UButton size="sm" variant="outline" @click="refreshViews()">Retry</UButton>
                </template>
              </UAlert>
              <div v-else class="py-3">
                <div class="flex items-center justify-between pb-2">
                  <p class="text-sm text-muted">{{ (views || []).length }} views</p>
                  <UButton size="sm" icon="i-lucide-plus" @click="openAddView">Add view</UButton>
                </div>
                <div v-if="!(views || []).length" class="py-8 text-center text-sm text-muted">
                  No views yet. Save the current list filter as a view.
                </div>
                <div v-else class="space-y-2">
                  <div v-for="view in views || []" :key="view.id" class="flex items-center justify-between gap-4 rounded-lg border border-default px-4 py-3">
                    <div>
                      <p class="text-sm font-medium">{{ view.name }}</p>
                      <p class="font-mono text-xs text-muted">{{ view.id }}</p>
                    </div>
                    <div class="flex items-center gap-1">
                      <UButton size="xs" variant="ghost" :to="`/app/${encodeURIComponent(selectedId)}?view=${encodeURIComponent(view.id)}`">Open</UButton>
                      <UButton size="xs" variant="ghost" color="error" @click="confirmDeleteView(view)">Delete</UButton>
                    </div>
                  </div>
                </div>
              </div>
            </template>
            <template #form-layout>
              <div v-if="formLayoutStatus === 'pending'" class="py-6">
                <USkeleton v-for="index in 2" :key="index" class="mb-3 h-8 w-full" />
              </div>
              <UAlert
                v-else-if="formLayoutStatus === 'error'"
                color="error"
                title="Cannot load form layout"
                :description="formLayoutError?.message || 'Check the Rust core connection.'"
              >
                <template #actions>
                  <UButton size="sm" variant="outline" @click="refreshFormLayout()">Retry</UButton>
                </template>
              </UAlert>
              <div v-else class="space-y-4 py-3">
                <div class="flex items-center justify-between">
                  <p class="text-sm text-muted">{{ layoutSections.length }} sections · {{ unassignedFields.length }} unassigned fields</p>
                  <div class="flex gap-2">
                    <UButton size="sm" variant="outline" icon="i-lucide-plus" @click="openAddSection">Add section</UButton>
                    <UButton size="sm" :loading="savingLayout" :disabled="!layoutDirty" @click="saveLayout">Save layout</UButton>
                  </div>
                </div>
                <UAlert v-if="layoutError" color="error" :title="layoutError" />
                <UBadge v-if="layoutDirty" color="warning" variant="subtle">Unsaved changes</UBadge>
                <div v-if="!layoutSections.length" class="py-8 text-center text-sm text-muted">
                  No sections yet. Add a section, then assign fields to it. Unassigned fields render under “Other”.
                </div>
                <UCard v-for="(section, sectionIndex) in layoutSections" :key="section.id">
                  <template #header>
                    <div class="flex items-center justify-between">
                      <p class="text-sm font-semibold">{{ section.label }} <span class="font-mono text-xs text-muted">{{ section.id }}</span></p>
                      <div class="flex gap-1">
                        <UButton size="xs" variant="ghost" icon="i-lucide-arrow-up" :disabled="sectionIndex === 0" @click="moveSection(sectionIndex, -1)" />
                        <UButton size="xs" variant="ghost" icon="i-lucide-arrow-down" :disabled="sectionIndex === layoutSections.length - 1" @click="moveSection(sectionIndex, 1)" />
                        <UButton size="xs" variant="ghost" color="error" @click="removeSection(sectionIndex)">Remove</UButton>
                      </div>
                    </div>
                  </template>
                  <div v-if="!section.fields.length" class="py-2 text-sm text-muted">No fields in this section yet.</div>
                  <div v-else class="space-y-1">
                    <div v-for="(fieldId, fieldIndex) in section.fields" :key="fieldId" class="flex items-center justify-between gap-2 rounded border border-default px-3 py-1.5">
                      <span class="font-mono text-sm">{{ fieldName(fieldId) }}</span>
                      <div class="flex gap-1">
                        <UButton size="xs" variant="ghost" icon="i-lucide-arrow-up" :disabled="fieldIndex === 0" @click="moveField(sectionIndex, fieldIndex, -1)" />
                        <UButton size="xs" variant="ghost" icon="i-lucide-arrow-down" :disabled="fieldIndex === section.fields.length - 1" @click="moveField(sectionIndex, fieldIndex, 1)" />
                        <USelectMenu
                          :model-value="String(sectionIndex)"
                          :items="layoutSections.map((s, i) => ({ label: s.label, value: String(i) }))"
                          value-key="value"
                          size="xs"
                          class="w-28"
                          aria-label="Move field to section"
                          @update:model-value="(value: string) => moveFieldToSection(sectionIndex, fieldIndex, Number(value))"
                        />
                        <UButton size="xs" variant="ghost" color="error" @click="removeFieldFromLayout(sectionIndex, fieldIndex)">Remove</UButton>
                      </div>
                    </div>
                  </div>
                  <div class="mt-2 flex items-center gap-2">
                    <USelectMenu
                      :model-value="''"
                      :items="unassignedFields.map(f => ({ label: f.name, value: f.id }))"
                      value-key="value"
                      placeholder="Add field…"
                      size="xs"
                      class="w-48"
                      @update:model-value="(value: string) => addFieldToSection(sectionIndex, value)"
                    />
                  </div>
                </UCard>
                <UCard>
                  <template #header>
                    <p class="text-sm font-semibold">Preview</p>
                  </template>
                  <div v-if="!layoutPreview.length" class="py-2 text-sm text-muted">Nothing to preview yet.</div>
                  <div v-else class="space-y-4">
                    <div v-for="section in layoutPreview" :key="section.id">
                      <p class="mb-1 text-xs font-semibold uppercase text-muted">{{ section.label }}</p>
                      <div class="space-y-2 rounded-lg border border-default p-3">
                        <div v-for="field in section.fields" :key="field.id" class="flex items-center justify-between gap-2 text-sm">
                          <span class="font-mono">{{ field.name }}</span>
                          <span class="text-xs text-muted">{{ field.type }}{{ field.required ? ' · required' : '' }}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                </UCard>
              </div>
            </template>
            <template #notifications>
              <div v-if="notifyRulesStatus === 'pending'" class="py-6">
                <USkeleton v-for="index in 2" :key="index" class="mb-3 h-8 w-full" />
              </div>
              <UAlert
                v-else-if="notifyRulesStatus === 'error'"
                color="error"
                title="Cannot load notification rules"
                :description="notifyRulesError?.message || 'Check the Rust core connection.'"
              >
                <template #actions>
                  <UButton size="sm" variant="outline" @click="refreshNotifyRules()">Retry</UButton>
                </template>
              </UAlert>
              <div v-else class="py-3">
                <div class="flex items-center justify-between pb-2">
                  <p class="text-sm text-muted">{{ (notifyRules || []).length }} rules · fires on workflow transition</p>
                  <UButton size="sm" icon="i-lucide-plus" @click="openAddRule">Add rule</UButton>
                </div>
                <div v-if="!(notifyRules || []).length" class="py-8 text-center text-sm text-muted">
                  No rules yet. Add a webhook URL to notify on every transition.
                </div>
                <div v-else class="space-y-2">
                  <div v-for="rule in notifyRules || []" :key="rule.id" class="flex items-center justify-between gap-4 rounded-lg border border-default px-4 py-3">
                    <div class="min-w-0">
                      <p class="truncate font-mono text-sm font-medium">{{ rule.target_url }}</p>
                      <p class="text-xs text-muted">{{ rule.trigger }} · {{ rule.active ? 'active' : 'disabled' }}</p>
                    </div>
                    <div class="flex items-center gap-2">
                      <USwitch
                        :model-value="rule.active"
                        :disabled="togglingRuleId === rule.id"
                        aria-label="Toggle rule active"
                        @update:model-value="toggleRule(rule)"
                      />
                      <UButton size="xs" variant="ghost" color="error" @click="confirmDeleteRule(rule)">Delete</UButton>
                    </div>
                  </div>
                </div>
              </div>
            </template>
          </UTabs>
        </UCard>
      </div>

      <!-- Create entity modal -->
      <UModal v-model:open="createOpen" title="New Entity">
        <template #body>
          <UForm class="space-y-4" @submit="createEntity">
            <UFormField label="ID" hint="lowercase, no spaces (e.g. work_order)">
              <UInput v-model="createForm.id" placeholder="e.g. customer" />
            </UFormField>
            <UFormField label="Name">
              <UInput v-model="createForm.name" placeholder="e.g. customer" />
            </UFormField>
            <UFormField label="Label">
              <UInput v-model="createForm.label" placeholder="e.g. Customer" />
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
            <UFormField label="Module" hint="Groups entities in the sidebar, e.g. Stock">
              <UInput v-model="editForm.module" placeholder="e.g. Stock" />
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
            <UFormField v-if="fieldForm.type === 'select'" label="Status field" hint="Drives workflow transitions and status badges">
              <USwitch v-model="fieldForm.is_status" />
            </UFormField>
            <UFormField v-if="fieldForm.type === 'reference'" label="Target entity" hint="Documents to pick from">
              <USelectMenu v-model="fieldForm.ref_entity" :items="(entities || []).map(e => ({ label: e.label, value: e.id }))" value-key="value" placeholder="Select entity…" class="w-full" />
            </UFormField>
            <UFormField v-if="fieldForm.type === 'computed'" label="Expression" hint="Template with {field} placeholders, e.g. {title} - {sku}">
              <UInput v-model="fieldForm.computed_expr" placeholder="{title} - {sku}" />
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

      <!-- State modal -->
      <UModal v-model:open="stateOpen" :title="editingState ? 'Edit state' : 'Add state'">
        <template #body>
          <UForm class="space-y-4" @submit="saveState">
            <UFormField label="Name" hint="lowercase, no spaces (e.g. open)">
              <UInput v-model="stateForm.name" placeholder="open" :disabled="!!editingState" />
            </UFormField>
            <UFormField label="Label">
              <UInput v-model="stateForm.label" placeholder="Open" />
            </UFormField>
            <UAlert v-if="stateError" color="error" :title="stateError" />
          </UForm>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="stateOpen = false">Cancel</UButton>
            <UButton :loading="savingState" @click="saveState">{{ editingState ? 'Save' : 'Create state' }}</UButton>
          </div>
        </template>
      </UModal>

      <!-- Delete state modal -->
      <UModal v-model:open="deleteStateOpen" title="Delete state">
        <template #body>
          <p class="text-sm text-muted">
            This will permanently delete the state
            <span class="font-mono">{{ stateToDelete?.name }}</span>. States used by transitions cannot be deleted.
          </p>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="deleteStateOpen = false">Cancel</UButton>
            <UButton color="error" :loading="deletingState" @click="removeState">Delete</UButton>
          </div>
        </template>
      </UModal>

      <!-- Transition modal -->
      <UModal v-model:open="transitionOpen" title="Add transition">
        <template #body>
          <UForm class="space-y-4" @submit="saveTransition">
            <UFormField label="From state">
              <USelectMenu
                v-model="transitionForm.from_state"
                :items="(workflow?.states || []).map(s => ({ label: s.label, value: s.name }))"
                value-key="value"
                placeholder="Select…"
                class="w-full"
              />
            </UFormField>
            <UFormField label="To state">
              <USelectMenu
                v-model="transitionForm.to_state"
                :items="(workflow?.states || []).map(s => ({ label: s.label, value: s.name }))"
                value-key="value"
                placeholder="Select…"
                class="w-full"
              />
            </UFormField>
            <UFormField label="Action" hint="lowercase, no spaces (e.g. submit)">
              <UInput v-model="transitionForm.action" placeholder="submit" />
            </UFormField>
            <UAlert v-if="transitionError" color="error" :title="transitionError" />
          </UForm>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="transitionOpen = false">Cancel</UButton>
            <UButton :loading="savingTransition" @click="saveTransition">Create transition</UButton>
          </div>
        </template>
      </UModal>

      <!-- Delete transition modal -->
      <UModal v-model:open="deleteTransitionOpen" title="Delete transition">
        <template #body>
          <p class="text-sm text-muted">
            This will permanently delete the transition
            <span class="font-mono">{{ transitionToDelete?.from_state }} → {{ transitionToDelete?.to_state }}</span>
            ({{ transitionToDelete?.action }}). This action cannot be undone.
          </p>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="deleteTransitionOpen = false">Cancel</UButton>
            <UButton color="error" :loading="deletingTransition" @click="removeTransition">Delete</UButton>
          </div>
        </template>
      </UModal>

      <!-- Add view modal -->
      <UModal v-model:open="viewOpen" title="Add view">
        <template #body>
          <UForm class="space-y-4" @submit="saveView">
            <UFormField label="Name" hint="e.g. Open only">
              <UInput v-model="viewForm.name" placeholder="Open only" />
            </UFormField>
            <UAlert v-if="viewError" color="error" :title="viewError" />
          </UForm>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="viewOpen = false">Cancel</UButton>
            <UButton :loading="savingView" @click="saveView">Create view</UButton>
          </div>
        </template>
      </UModal>

      <!-- Add section modal -->
      <UModal v-model:open="sectionOpen" title="Add section">
        <template #body>
          <UForm class="space-y-4" @submit="saveSection">
            <UFormField label="Label" hint="e.g. Main details">
              <UInput v-model="sectionForm.label" placeholder="Main details" />
            </UFormField>
            <UAlert v-if="layoutError" color="error" :title="layoutError" />
          </UForm>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="sectionOpen = false">Cancel</UButton>
            <UButton @click="saveSection">Add section</UButton>
          </div>
        </template>
      </UModal>

      <!-- Add notification rule modal -->
      <UModal v-model:open="ruleOpen" title="Add notification rule">
        <template #body>
          <UForm class="space-y-4" @submit="saveRule">
            <UFormField label="Webhook URL" hint="https://… — POSTed on every transition">
              <UInput v-model="ruleForm.target_url" placeholder="https://example.com/hook" />
            </UFormField>
            <UFormField label="Active">
              <USwitch v-model="ruleForm.active" />
            </UFormField>
            <UAlert v-if="ruleError" color="error" :title="ruleError" />
          </UForm>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="ruleOpen = false">Cancel</UButton>
            <UButton :loading="savingRule" @click="saveRule">Create rule</UButton>
          </div>
        </template>
      </UModal>

      <!-- Delete notification rule modal -->
      <UModal v-model:open="deleteRuleOpen" title="Delete rule">
        <template #body>
          <p class="text-sm text-muted">
            This will permanently delete the webhook rule
            <span class="font-mono">{{ ruleToDelete?.target_url }}</span>. This action cannot be undone.
          </p>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="deleteRuleOpen = false">Cancel</UButton>
            <UButton color="error" :loading="deletingRule" @click="removeRule">Delete</UButton>
          </div>
        </template>
      </UModal>

      <!-- Delete view modal -->
      <UModal v-model:open="deleteViewOpen" title="Delete view">
        <template #body>
          <p class="text-sm text-muted">
            This will permanently delete the view
            <span class="font-mono">{{ viewToDelete?.name }}</span>. This action cannot be undone.
          </p>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="deleteViewOpen = false">Cancel</UButton>
            <UButton color="error" :loading="deletingView" @click="removeView">Delete</UButton>
          </div>
        </template>
      </UModal>
    </template>
  </UDashboardPanel>
</template>