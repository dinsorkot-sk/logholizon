<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

type SolutionEntity = { name: string; label: string; fields: number }
type Solution = {
  key: string
  name: string
  label: string
  description: string
  icon: string
  color: string
  version: string
  entities: SolutionEntity[]
  relations: number
  actions: number
  automations: number
  dashboards: string[]
}
type InstalledModule = { id: string; name: string; label: string; status: string; semantic_version: string }
type PackagePreview = {
  valid: boolean
  action: string
  name: string
  version: string
  conflict: string | null
  migration: { creates: string[]; updates: string[]; warnings: string[] }
  dependencies: { name: string; required_version: string; installed_version: string | null; satisfied: boolean }[]
}

const toast = useToast()
const { data: solutions, status, error, refresh } = await useFetch<Solution[]>('/api/solutions')
const { data: modules, refresh: refreshModules } = await useFetch<InstalledModule[]>('/api/modules')

const search = ref('')
const previewOpen = ref(false)
const previewSolution = ref<Solution | null>(null)
const preview = ref<PackagePreview | null>(null)
const previewLoading = ref(false)
const previewError = ref('')
const installingKey = ref('')

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase()
  const list = solutions.value || []
  if (!q) return list
  return list.filter(s =>
    s.name.toLowerCase().includes(q)
    || s.label.toLowerCase().includes(q)
    || s.description.toLowerCase().includes(q)
  )
})

function installedModule(solution: Solution) {
  return (modules.value || []).find(m => m.name === solution.name)
}

function statusColor(value: string) {
  switch (value) {
    case 'enabled': return 'success'
    case 'published': return 'success'
    case 'archived': return 'neutral'
    default: return 'warning'
  }
}

async function openPreview(solution: Solution) {
  previewSolution.value = solution
  preview.value = null
  previewError.value = ''
  previewOpen.value = true
  previewLoading.value = true
  try {
    const pkg = await $fetch<Record<string, unknown>>(`/api/solutions/${encodeURIComponent(solution.key)}`)
    preview.value = await $fetch<PackagePreview>('/api/modules/packages/preview', {
      method: 'POST',
      body: { package: pkg }
    })
  } catch (e: any) {
    previewError.value = e?.data?.message || e?.statusMessage || 'Failed to preview package'
  } finally {
    previewLoading.value = false
  }
}

async function install(solution: Solution) {
  installingKey.value = solution.key
  try {
    const pkg = await $fetch<Record<string, unknown>>(`/api/solutions/${encodeURIComponent(solution.key)}`)
    const installed = await $fetch<InstalledModule>('/api/modules/packages/install', {
      method: 'POST',
      body: { package: pkg }
    })
    previewOpen.value = false
    await refreshModules()
    toast.add({ title: `${solution.label} installed`, description: `${installed.id} · ${installed.status}`, color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    toast.add({
      title: `Unable to install ${solution.label}`,
      description: e?.data?.message || e?.statusMessage || 'Install failed',
      color: 'error',
      icon: 'i-lucide-alert-circle'
    })
  } finally {
    installingKey.value = ''
  }
}
</script>

<template>
  <UDashboardPanel id="solutions">
    <template #header>
      <UDashboardNavbar title="Solution Library">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
        <template #right>
          <UButton variant="ghost" icon="i-lucide-refresh-cw" :loading="status === 'pending'" @click="refresh()">Refresh</UButton>
        </template>
      </UDashboardNavbar>
    </template>
    <template #body>
      <div class="mx-auto max-w-5xl space-y-4">
        <p class="text-sm text-muted">
          Ready-made ERP solutions ship as module packages. Preview the migration plan,
          then install — packages run through the same lifecycle as user-created modules.
        </p>
        <UAlert v-if="error" color="error" title="Cannot load solutions" :description="error.message">
          <template #actions>
            <UButton size="sm" variant="outline" @click="refresh()">Retry</UButton>
          </template>
        </UAlert>
        <UInput v-model="search" placeholder="Search solutions…" icon="i-lucide-search" class="max-w-sm" />
        <div v-if="status === 'pending'" class="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <USkeleton v-for="index in 4" :key="index" class="h-44 w-full" />
        </div>
        <div v-else-if="!filtered.length" class="flex flex-col items-center gap-3 py-16 text-center">
          <UIcon name="i-lucide-package-open" class="h-10 w-10 text-muted" />
          <p class="text-sm text-muted">No solutions match your search.</p>
        </div>
        <div v-else class="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <UCard v-for="solution in filtered" :key="solution.key">
            <template #header>
              <div class="flex items-center gap-3">
                <UIcon :name="solution.icon" class="h-6 w-6" />
                <div class="min-w-0 flex-1">
                  <h2 class="truncate font-semibold">{{ solution.label }}</h2>
                  <p class="font-mono text-xs text-muted">{{ solution.name }} · v{{ solution.version }}</p>
                </div>
                <UBadge v-if="installedModule(solution)" :color="statusColor(installedModule(solution)!.status)" variant="subtle">
                  {{ installedModule(solution)!.status }}
                </UBadge>
                <UBadge v-else color="neutral" variant="subtle">not installed</UBadge>
              </div>
            </template>
            <p class="line-clamp-2 min-h-10 text-sm text-muted">{{ solution.description }}</p>
            <div class="mt-3 flex flex-wrap gap-1">
              <UBadge v-for="entity in solution.entities" :key="entity.name" color="neutral" variant="outline">
                {{ entity.label }} ({{ entity.fields }})
              </UBadge>
            </div>
            <p class="mt-2 text-xs text-muted">
              {{ solution.relations }} relations · {{ solution.actions }} actions ·
              {{ solution.automations }} automations · {{ solution.dashboards.length }} dashboards
            </p>
            <template #footer>
              <div class="flex justify-end gap-2">
                <UButton size="sm" variant="outline" @click="openPreview(solution)">Preview</UButton>
                <UButton
                  size="sm"
                  :loading="installingKey === solution.key"
                  :disabled="!!installedModule(solution)"
                  @click="install(solution)"
                >
                  {{ installedModule(solution) ? 'Installed' : 'Install' }}
                </UButton>
              </div>
            </template>
          </UCard>
        </div>
      </div>

      <!-- Preview modal -->
      <UModal v-model:open="previewOpen" :title="previewSolution ? `Preview ${previewSolution.label}` : 'Preview'">
        <template #body>
          <div v-if="previewLoading" class="space-y-2 py-4">
            <USkeleton class="h-6 w-full" />
            <USkeleton class="h-6 w-2/3" />
          </div>
          <UAlert v-else-if="previewError" color="error" :title="previewError" />
          <div v-else-if="preview" class="space-y-3">
            <div class="flex items-center gap-2">
              <UBadge :color="preview.valid ? 'success' : 'error'" variant="subtle">
                {{ preview.valid ? 'ready to install' : preview.action }}
              </UBadge>
              <span class="font-mono text-xs text-muted">v{{ preview.version }}</span>
            </div>
            <UAlert v-if="preview.conflict" color="warning" :title="preview.conflict" />
            <div v-if="preview.migration.creates.length">
              <p class="text-sm font-medium">Creates ({{ preview.migration.creates.length }})</p>
              <div class="mt-1 flex flex-wrap gap-1">
                <UBadge v-for="name in preview.migration.creates" :key="name" color="success" variant="outline">{{ name }}</UBadge>
              </div>
            </div>
            <div v-if="preview.migration.updates.length">
              <p class="text-sm font-medium">Updates ({{ preview.migration.updates.length }})</p>
              <div class="mt-1 flex flex-wrap gap-1">
                <UBadge v-for="name in preview.migration.updates" :key="name" color="warning" variant="outline">{{ name }}</UBadge>
              </div>
            </div>
            <div v-if="preview.migration.warnings.length">
              <p class="text-sm font-medium">Warnings</p>
              <ul class="list-disc pl-5 text-sm text-muted">
                <li v-for="(warning, index) in preview.migration.warnings" :key="index">{{ warning }}</li>
              </ul>
            </div>
            <div v-if="preview.dependencies.length">
              <p class="text-sm font-medium">Dependencies</p>
              <ul class="space-y-1 text-sm">
                <li v-for="dep in preview.dependencies" :key="dep.name" class="flex items-center gap-2">
                  <UIcon :name="dep.satisfied ? 'i-lucide-check' : 'i-lucide-x'" :class="dep.satisfied ? 'text-success' : 'text-error'" />
                  <span class="font-mono">{{ dep.name }}</span>
                  <span class="text-muted">{{ dep.required_version }}</span>
                </li>
              </ul>
            </div>
          </div>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="previewOpen = false">Close</UButton>
            <UButton
              v-if="previewSolution"
              :loading="installingKey === previewSolution.key"
              :disabled="!preview?.valid || !!installedModule(previewSolution)"
              @click="install(previewSolution)"
            >
              Install
            </UButton>
          </div>
        </template>
      </UModal>
    </template>
  </UDashboardPanel>
</template>
