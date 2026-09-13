<script setup lang="ts">
// Module-scoped entity route. Module entities materialize as first-class
// entities (`<module>_<entity>`) on publish, so the canonical entity runtime
// page serves them with full parity (workflow, bulk, import/export, detail
// timeline, comments, attachments, audit). This route resolves the module
// entity to its materialized ID and redirects there instead of maintaining
// a second, thinner runtime.
definePageMeta({ middleware: 'auth' })

type Entity = { id: string; name: string; label: string; module?: string | null }

const route = useRoute()
const moduleName = computed(() => String(route.params.module || ''))
const entityName = computed(() => String(route.params.entity || ''))
const { data: entities } = await useFetch<Entity[]>('/api/entities')
const entityId = computed(() =>
  (entities.value || []).find(
    e => e.name === entityName.value && String(e.module || '') === moduleName.value
  )?.id || ''
)

watch(entityId, (id) => {
  if (id) navigateTo(`/app/${encodeURIComponent(id)}`, { replace: true })
}, { immediate: true })
</script>

<template>
  <UDashboardPanel :id="`dynamic-${moduleName}-${entityName}`">
    <template #header>
      <UDashboardNavbar :title="entityName">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
      </UDashboardNavbar>
    </template>
    <template #body>
      <div class="mx-auto max-w-5xl space-y-4">
        <UBreadcrumb :items="[{ label: 'Apps', to: '/apps' }, { label: moduleName, to: `/app/modules/${encodeURIComponent(moduleName)}` }, { label: entityName }]" />
        <UAlert
          v-if="!entityId"
          color="error"
          title="Entity not found"
          description="The module or entity is not available to your account."
        />
        <div v-else class="flex items-center gap-2 text-sm text-muted">
          <UIcon name="i-lucide-loader-circle" class="animate-spin" />
          Opening {{ entityName }}…
        </div>
      </div>
    </template>
  </UDashboardPanel>
</template>
