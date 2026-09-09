<script setup lang="ts">
const route = useRoute()
const dashboard = ref<any>(null)
const output = ref<any[]>([])
const loading = ref(true)
async function load() {
  loading.value = true
  try {
    dashboard.value = await $fetch(`/api/dashboards/${route.params.id}`)
    output.value = await $fetch(`/api/dashboards/${route.params.id}`, { method: 'POST' })
  } finally { loading.value = false }
}
await load()
</script>

<template>
  <div class="p-6 space-y-6">
    <div><h1 class="text-2xl font-semibold">{{ dashboard?.name }}</h1><p class="text-muted">{{ dashboard?.description }}</p></div>
    <div v-if="loading" class="text-muted">Loading dashboard…</div>
    <div v-else class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
      <UCard v-for="item in output" :key="item.widget.id" class="min-h-40">
        <div class="font-medium">{{ item.widget.title }}</div>
        <div class="mt-3 text-sm text-muted">{{ item.widget.kind }} · {{ item.widget.entity_id }}</div>
        <pre class="mt-3 max-h-64 overflow-auto text-xs">{{ JSON.stringify(item.result, null, 2) }}</pre>
      </UCard>
    </div>
  </div>
</template>
