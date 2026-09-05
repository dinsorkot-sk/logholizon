<script setup lang="ts">
type Count = { status: string; count: number }
const entityId = ref('work_order')
const { data: counts, status, error, refresh } = await useFetch<Count[]>(
  () => `/api/dashboard/counts?entity_id=${encodeURIComponent(entityId.value)}`,
  { watch: [entityId] }
)
</script>

<template>
  <UContainer class="py-8">
    <div class="mb-6 flex items-center justify-between">
      <h1 class="text-xl font-semibold">Dashboard</h1>
      <UButton variant="ghost" :loading="status === 'pending'" @click="refresh()">Refresh</UButton>
    </div>
    <UAlert v-if="error" color="error" title="Cannot load dashboard" :description="error.message" />
    <div v-else class="grid grid-cols-1 gap-4 sm:grid-cols-3">
      <UCard v-for="item in counts || []" :key="item.status">
        <p class="text-sm text-gray-500">{{ item.status }}</p>
        <p class="text-3xl font-semibold">{{ item.count }}</p>
      </UCard>
    </div>
  </UContainer>
</template>
