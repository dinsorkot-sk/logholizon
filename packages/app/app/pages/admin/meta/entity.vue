<script setup lang="ts">
type Entity = { id: string; name: string; label: string }

const { data: entities, status, refresh } = await useFetch<Entity[]>('/api/meta/entities')

const form = reactive({ id: '', name: '', label: '' })
const creating = ref(false)
const error = ref('')
const toast = useToast()

async function createEntity() {
  error.value = ''
  if (!form.id.trim() || !form.name.trim() || !form.label.trim()) {
    error.value = 'id, name, and label are required'
    return
  }
  creating.value = true
  try {
    await $fetch('/api/meta/entities', { method: 'POST', body: { ...form } })
    form.id = ''
    form.name = ''
    form.label = ''
    await refresh()
    toast.add({ title: 'Entity created', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    error.value = e?.data?.message || e?.statusMessage || 'Failed to create entity'
    toast.add({ title: 'Unable to create entity', description: error.value, color: 'error', icon: 'i-lucide-alert-circle' })
  } finally {
    creating.value = false
  }
}
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
    <UCard>

      <UAlert
        v-if="status === 'error'"
        color="error"
        title="Cannot load entities"
        description="Is the Rust core running on port 8787?"
        class="mb-4"
      />

      <div v-else-if="status === 'pending'" class="py-8 text-center text-sm text-gray-500">
        Loading entities…
      </div>

      <template v-else>
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b text-left text-gray-500">
              <th class="py-2 pr-4">ID</th>
              <th class="py-2 pr-4">Name</th>
              <th class="py-2">Label</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="entity in entities" :key="entity.id" class="border-b last:border-0">
              <td class="py-2 pr-4 font-mono">{{ entity.id }}</td>
              <td class="py-2 pr-4">{{ entity.name }}</td>
              <td class="py-2">{{ entity.label }}</td>
            </tr>
            <tr v-if="!entities?.length">
              <td colspan="3" class="py-8 text-center text-gray-500">
                No entities yet. Create one below.
              </td>
            </tr>
          </tbody>
        </table>

        <UForm class="mt-6 border-t pt-6" @submit="createEntity">
          <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
            <UFormField label="ID">
              <UInput v-model="form.id" placeholder="work_order" />
            </UFormField>
            <UFormField label="Name">
              <UInput v-model="form.name" placeholder="work_order" />
            </UFormField>
            <UFormField label="Label">
              <UInput v-model="form.label" placeholder="Work Order" />
            </UFormField>
          </div>
          <UAlert v-if="error" color="error" :title="error" class="mt-4" />
          <div class="mt-4">
            <UButton type="submit" :loading="creating">
              Create Entity
            </UButton>
          </div>
        </UForm>
      </template>
    </UCard>
    </template>
  </UDashboardPanel>
</template>