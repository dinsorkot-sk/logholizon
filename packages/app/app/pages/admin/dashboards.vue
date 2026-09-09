<script setup lang="ts">
type CoreDashboard = { id: string; name: string; description: string; layout: unknown[]; filters: Record<string, unknown>; roles: string[]; users: string[]; active: boolean }
const dashboards = ref<CoreDashboard[]>([])
const selected = ref<CoreDashboard | null>(null)
const saving = ref(false)
const form = reactive({ name: '', description: '', active: true, roles: '', users: '', layout: '[]', filters: '{}' })
async function load() { dashboards.value = await $fetch<CoreDashboard[]>('/api/meta/dashboards') }
function select(d: CoreDashboard) { selected.value = d; Object.assign(form, { name: d.name, description: d.description, active: d.active, roles: d.roles.join(','), users: d.users.join(','), layout: JSON.stringify(d.layout, null, 2), filters: JSON.stringify(d.filters, null, 2) }) }
function reset() { selected.value = null; Object.assign(form, { name: '', description: '', active: true, roles: '', users: '', layout: '[]', filters: '{}' }) }
async function save() {
  saving.value = true
  try {
    const payload = { name: form.name, description: form.description, active: form.active, roles: form.roles.split(',').map(x => x.trim()).filter(Boolean), users: form.users.split(',').map(x => x.trim()).filter(Boolean), layout: JSON.parse(form.layout), filters: JSON.parse(form.filters) }
    if (selected.value) await $fetch(`/api/meta/dashboards/${selected.value.id}`, { method: 'PUT', body: payload })
    else await $fetch('/api/meta/dashboards', { method: 'POST', body: payload })
    await load(); reset()
  } finally { saving.value = false }
}
async function remove(id: string) { await $fetch(`/api/meta/dashboards/${id}`, { method: 'DELETE' }); await load(); if (selected.value?.id === id) reset() }
await load()
</script>

<template>
  <div class="p-6 space-y-6">
    <div class="flex items-center justify-between"><div><h1 class="text-2xl font-semibold">Dashboard Builder</h1><p class="text-muted">Generic dashboards powered by saved report configurations.</p></div><UButton label="New dashboard" @click="reset" /></div>
    <div class="grid gap-6 lg:grid-cols-[280px_1fr]">
      <UCard><div class="space-y-2"><button v-for="d in dashboards" :key="d.id" class="w-full rounded px-3 py-2 text-left hover:bg-muted" @click="select(d)"><div class="font-medium">{{ d.name }}</div><div class="text-xs text-muted">{{ d.layout.length }} widgets</div></button></div></UCard>
      <UCard><div class="grid gap-4 md:grid-cols-2"><UInput v-model="form.name" placeholder="Dashboard name" /><UInput v-model="form.description" placeholder="Description" /><UInput v-model="form.roles" placeholder="Roles, comma separated" /><UInput v-model="form.users" placeholder="Users, comma separated" /><UTextarea v-model="form.filters" :rows="5" placeholder="Dashboard filters JSON" /><UTextarea v-model="form.layout" :rows="14" placeholder="Widget grid JSON" /></div><div class="mt-4 flex gap-2"><UCheckbox v-model="form.active" label="Active" /><UButton :loading="saving" label="Save" @click="save" /><UButton v-if="selected" color="error" variant="soft" label="Delete" @click="remove(selected.id)" /></div></UCard>
    </div>
  </div>
</template>

