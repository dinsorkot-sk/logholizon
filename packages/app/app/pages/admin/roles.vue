<script setup lang="ts">

definePageMeta({ middleware: 'auth' })

type Role = { id: string; name: string; label: string; description: string; system: boolean; created_at: string }
const toast = useToast()
const { data: roles, status, refresh } = await useFetch<Role[]>('/api/admin/roles')
const open = ref(false)
const saving = ref(false)
const error = ref('')
const form = reactive({ name: '', label: '', description: '' })

function openCreate() {
  form.name = ''
  form.label = ''
  form.description = ''
  error.value = ''
  open.value = true
}

async function createRole() {
  error.value = ''
  if (!form.name.trim() || !form.label.trim()) {
    error.value = 'Role name and label are required'
    return
  }
  saving.value = true
  try {
    await $fetch('/api/admin/roles', { method: 'POST', body: { ...form } })
    open.value = false
    await refresh()
    toast.add({ title: 'Role created', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    error.value = e?.data?.message || e?.statusMessage || 'Failed to create role'
  } finally { saving.value = false }
}

async function removeRole(role: Role) {
  if (role.system) return
  try {
    await $fetch(`/api/admin/roles/${encodeURIComponent(role.id)}`, { method: 'DELETE' as any })
    await refresh()
    toast.add({ title: 'Role deleted', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    toast.add({ title: 'Unable to delete role', description: e?.data?.message || e?.statusMessage || 'Delete failed', color: 'error' })
  }
}
</script>

<template>
  <UDashboardPanel id="roles">
    <template #header>
      <UDashboardNavbar title="Roles">
        <template #leading><UDashboardSidebarCollapse /></template>
        <template #right><UButton icon="i-lucide-plus" @click="openCreate">New role</UButton></template>
      </UDashboardNavbar>
    </template>
    <template #body>
      <div class="mx-auto max-w-4xl">
        <UCard>
          <div v-if="status === 'pending'" class="space-y-3"><USkeleton v-for="i in 3" :key="i" class="h-12 w-full" /></div>
          <div v-else class="divide-y divide-default">
            <div v-for="role in roles || []" :key="role.id" class="flex items-center justify-between gap-4 py-4">
              <div>
                <div class="flex items-center gap-2"><span class="font-medium">{{ role.label }}</span><UBadge variant="subtle">{{ role.name }}</UBadge><UBadge v-if="role.system" variant="outline">System</UBadge></div>
                <p class="mt-1 text-sm text-muted">{{ role.description || 'No description' }}</p>
              </div>
              <UButton v-if="!role.system" color="error" variant="ghost" icon="i-lucide-trash-2" @click="removeRole(role)">Delete</UButton>
            </div>
            <div v-if="!roles?.length" class="py-10 text-center text-muted">No roles yet.</div>
          </div>
        </UCard>
      </div>

      <UModal v-model:open="open" title="New role">
        <template #body>
          <UForm class="space-y-4" @submit="createRole">
            <UFormField label="Name" hint="lowercase letters, digits, _ or -"><UInput v-model="form.name" placeholder="sales_manager" /></UFormField>
            <UFormField label="Label"><UInput v-model="form.label" placeholder="Sales Manager" /></UFormField>
            <UFormField label="Description"><UTextarea v-model="form.description" /></UFormField>
            <UAlert v-if="error" color="error" :title="error" />
          </UForm>
        </template>
        <template #footer><div class="flex justify-end gap-2"><UButton variant="ghost" @click="open = false">Cancel</UButton><UButton :loading="saving" @click="createRole">Create role</UButton></div></template>
      </UModal>
    </template>
  </UDashboardPanel>
</template>

