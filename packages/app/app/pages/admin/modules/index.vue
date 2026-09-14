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
  created_at: string
  updated_at: string
}

const toast = useToast()
const { data: modules, status, error, refresh } = await useFetch<Module[]>('/api/modules')

const search = ref('')
const createOpen = ref(false)
const createForm = reactive({ name: '', label: '', description: '', icon: '', color: '' })
const creating = ref(false)
const createError = ref('')

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase()
  const list = modules.value || []
  if (!q) return list
  return list.filter(m => m.name.toLowerCase().includes(q) || m.label.toLowerCase().includes(q))
})

function statusColor(value: string) {
  switch (value) {
    case 'published': return 'success'
    case 'archived': return 'neutral'
    default: return 'warning'
  }
}

async function createModule() {
  createError.value = ''
  if (!createForm.name.trim() || !createForm.label.trim()) {
    createError.value = 'name and label are required (snake_case name)'
    return
  }
  creating.value = true
  try {
    await $fetch('/api/modules', {
      method: 'POST',
      body: {
        name: createForm.name.trim(),
        label: createForm.label.trim(),
        description: createForm.description.trim(),
        icon: createForm.icon.trim(),
        color: createForm.color.trim(),
        definition: { entities: [] }
      }
    })
    createOpen.value = false
    createForm.name = ''
    createForm.label = ''
    createForm.description = ''
    await refresh()
    toast.add({ title: 'Module created', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    createError.value = cause?.data?.message || cause?.statusMessage || 'Failed to create module'
  } finally {
    creating.value = false
  }
}
</script>

<template>
  <UDashboardPanel id="modules">
    <template #header>
      <UDashboardNavbar title="Module Builder">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
        <template #right>
          <UButton variant="ghost" icon="i-lucide-refresh-cw" :loading="status === 'pending'" @click="refresh()">Refresh</UButton>
          <UButton icon="i-lucide-plus" @click="createOpen = true">New module</UButton>
        </template>
      </UDashboardNavbar>
    </template>
    <template #body>
      <div class="mx-auto max-w-5xl space-y-4">
        <UAlert v-if="error" color="error" title="Cannot load modules" :description="error.message">
          <template #actions>
            <UButton size="sm" variant="outline" @click="refresh()">Retry</UButton>
          </template>
        </UAlert>
        <UInput v-model="search" placeholder="Search modules…" icon="i-lucide-search" class="max-w-sm" />
        <div v-if="status === 'pending'" class="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <USkeleton v-for="index in 4" :key="index" class="h-32 w-full" />
        </div>
        <div v-else-if="!filtered.length" class="flex flex-col items-center gap-3 py-16 text-center">
          <UIcon name="i-lucide-box" class="h-10 w-10 text-muted" />
          <p class="text-sm text-muted">No modules yet. Create your first user-defined module.</p>
          <UButton icon="i-lucide-plus" @click="createOpen = true">New module</UButton>
        </div>
        <div v-else class="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <UCard v-for="module in filtered" :key="module.id" :to="`/admin/modules/${encodeURIComponent(module.id)}`">
            <div class="flex items-start justify-between gap-2">
              <div class="min-w-0">
                <h2 class="truncate text-base font-semibold">{{ module.label }}</h2>
                <p class="truncate font-mono text-xs text-muted">{{ module.owner }}/{{ module.name }} · v{{ module.version }}</p>
                <p v-if="module.description" class="mt-1 line-clamp-2 text-sm text-muted">{{ module.description }}</p>
              </div>
              <UBadge :color="statusColor(module.status)" variant="subtle">{{ module.status }}</UBadge>
            </div>
            <p class="mt-3 text-xs text-muted">{{ (module.definition?.entities || []).length }} entities</p>
          </UCard>
        </div>
      </div>
      <UModal v-model:open="createOpen" title="New module">
        <template #body>
          <UForm class="space-y-4" @submit="createModule">
            <UFormField label="Name (snake_case)" required>
              <UInput v-model="createForm.name" placeholder="vehicle" />
            </UFormField>
            <UFormField label="Label" required>
              <UInput v-model="createForm.label" placeholder="Vehicle Management" />
            </UFormField>
            <UFormField label="Description">
              <UInput v-model="createForm.description" placeholder="Fleet, drivers, maintenance" />
            </UFormField>
            <UAlert v-if="createError" color="error" :title="createError" />
          </UForm>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="createOpen = false">Cancel</UButton>
            <UButton :loading="creating" @click="createModule">Create</UButton>
          </div>
        </template>
      </UModal>
    </template>
  </UDashboardPanel>
</template>
