<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

import { h, resolveComponent } from 'vue'
import type { TableColumn } from '@nuxt/ui'

const UButton = resolveComponent('UButton')
const UBadge = resolveComponent('UBadge')

type Entity = { id: string; name: string; label: string }
type WorkflowState = { id: string; name: string; label: string; position: number }
type WorkflowTransition = { id: string; action: string; from_state: string; to_state: string }
type WorkflowDefinition = { states: WorkflowState[]; transitions: WorkflowTransition[] }

const toast = useToast()
const { data: entities, status, refresh } = await useFetch<Entity[]>('/api/meta/entities')
const entityId = ref('')
watch(entities, (list) => {
  const first = list?.[0]
  if (list?.length && first && !entityId.value) entityId.value = first.id
}, { immediate: true })

const workflowUrl = computed(() => entityId.value ? `/api/meta/entities/${encodeURIComponent(entityId.value)}/workflow` : '')
const { data: workflow, status: workflowStatus, error: workflowError, refresh: refreshWorkflow } = await useFetch<WorkflowDefinition>(workflowUrl, { watch: [workflowUrl] })

const selectedEntity = computed(() => (entities.value || []).find(e => e.id === entityId.value))

// --- State CRUD ---
const stateOpen = ref(false)
const editingState = ref<WorkflowState | null>(null)
const stateForm = reactive({ name: '', label: '' })
const stateError = ref('')
const savingState = ref(false)
const deleteStateOpen = ref(false)
const stateToDelete = ref<WorkflowState | null>(null)
const deletingState = ref(false)

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
  if (!entityId.value) return
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
      await $fetch(`/api/meta/entities/${encodeURIComponent(entityId.value)}/workflow/states`, {
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
    toast.add({ title: 'Unable to delete state', description: e?.data?.message || e?.statusMessage || 'Delete failed', color: 'error', icon: 'i-lucide-alert-circle' })
  } finally {
    deletingState.value = false
  }
}

// --- Transition CRUD ---
const transitionOpen = ref(false)
const transitionForm = reactive({ from_state: '', to_state: '', action: '' })
const transitionError = ref('')
const savingTransition = ref(false)
const deleteTransitionOpen = ref(false)
const transitionToDelete = ref<WorkflowTransition | null>(null)
const deletingTransition = ref(false)

function openAddTransition() {
  transitionForm.from_state = ''
  transitionForm.to_state = ''
  transitionForm.action = ''
  transitionError.value = ''
  transitionOpen.value = true
}

async function saveTransition() {
  if (!entityId.value) return
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
    await $fetch(`/api/meta/entities/${encodeURIComponent(entityId.value)}/workflow/transitions`, {
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
    toast.add({ title: 'Unable to delete transition', description: e?.data?.message || e?.statusMessage || 'Delete failed', color: 'error', icon: 'i-lucide-alert-circle' })
  } finally {
    deletingTransition.value = false
  }
}

// --- Visual step list ---
const stateLabel = (name: string) => workflow.value?.states.find(s => s.name === name)?.label || name

const steps = computed(() => {
  const states = workflow.value?.states || []
  const transitions = workflow.value?.transitions || []
  return states.map((state, index) => ({
    state,
    isLast: index === states.length - 1,
    outgoing: transitions.filter(t => t.from_state === state.name)
  }))
})

const transitionColumns: TableColumn<WorkflowTransition>[] = [
  {
    accessorKey: 'action',
    header: 'Action',
    cell: ({ row }) => h(UBadge, { variant: 'subtle' }, () => row.original.action)
  },
  {
    accessorKey: 'to_state',
    header: 'To',
    cell: ({ row }) => h('span', { class: 'font-mono' }, stateLabel(row.original.to_state))
  },
  {
    id: 'actions',
    header: () => h('span', { class: 'sr-only' }, 'Actions'),
    cell: ({ row }) => h(UButton, { size: 'xs', variant: 'ghost', color: 'error', onClick: () => confirmDeleteTransition(row.original) }, () => 'Delete')
  }
]
</script>

<template>
  <UDashboardPanel id="workflow-builder">
    <template #header>
      <UDashboardNavbar title="Workflow Builder">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
        <template #right>
          <UButton variant="ghost" icon="i-lucide-refresh-cw" :loading="workflowStatus === 'pending'" @click="refreshWorkflow()">Refresh</UButton>
        </template>
      </UDashboardNavbar>
    </template>
    <template #body>
      <div class="mx-auto max-w-3xl space-y-4">
        <div class="flex flex-wrap items-center gap-2">
          <USelectMenu
            v-model="entityId"
            :items="(entities || []).map(e => ({ label: e.label, value: e.id }))"
            value-key="value"
            class="w-56"
            aria-label="Select entity"
          />
          <UButton size="sm" icon="i-lucide-plus" @click="openAddState">Add state</UButton>
          <UButton size="sm" variant="outline" icon="i-lucide-arrow-right-left" @click="openAddTransition">Add transition</UButton>
        </div>

        <UAlert v-if="workflowStatus === 'error'" color="error" title="Cannot load workflow" :description="workflowError?.message || 'Check the Rust core connection.'">
          <template #actions>
            <UButton size="sm" variant="outline" @click="refreshWorkflow()">Retry</UButton>
          </template>
        </UAlert>

        <div v-else-if="workflowStatus === 'pending'" class="space-y-3">
          <USkeleton v-for="index in 3" :key="index" class="h-16 w-full" />
        </div>

        <div v-else-if="!workflow?.states?.length" class="flex flex-col items-center gap-3 py-16 text-center">
          <UIcon name="i-lucide-git-branch" class="h-10 w-10 text-muted" />
          <p class="text-sm text-muted">No states yet for {{ selectedEntity?.label || entityId }}.</p>
          <UButton icon="i-lucide-plus" @click="openAddState">Add first state</UButton>
        </div>

        <div v-else class="space-y-0">
          <div v-for="step in steps" :key="step.state.id">
            <UCard class="w-full">
              <div class="flex items-center justify-between gap-4">
                <div class="flex items-center gap-3">
                  <span class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-primary/10 font-mono text-sm text-primary">
                    {{ step.state.position + 1 }}
                  </span>
                  <div>
                    <p class="font-mono text-sm font-medium">{{ step.state.name }}</p>
                    <p class="text-xs text-muted">{{ step.state.label }}</p>
                  </div>
                </div>
                <div class="flex items-center gap-1">
                  <UBadge v-if="step.outgoing.length" variant="subtle">{{ step.outgoing.length }} transition{{ step.outgoing.length > 1 ? 's' : '' }}</UBadge>
                  <UButton size="xs" variant="ghost" icon="i-lucide-pencil" @click="openEditState(step.state)" />
                  <UButton size="xs" variant="ghost" color="error" icon="i-lucide-trash" @click="confirmDeleteState(step.state)" />
                </div>
              </div>
              <div v-if="step.outgoing.length" class="mt-3">
                <UTable :data="step.outgoing" :columns="transitionColumns" :get-row-id="(row: WorkflowTransition) => row.id" class="w-full" />
              </div>
            </UCard>
            <div v-if="!step.isLast" class="flex justify-center py-1">
              <UIcon name="i-lucide-arrow-down" class="h-4 w-4 text-muted" />
            </div>
          </div>
        </div>
      </div>

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
    </template>
  </UDashboardPanel>
</template>