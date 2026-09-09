<script setup lang="ts">
type Field = { id: string; name: string; label?: string; hidden?: boolean; sortable?: boolean }
type Row = { id: string; payload: Record<string, unknown> }
const props = defineProps<{ fields: Field[]; rows: Row[]; total: number; selected?: Set<string> }>()
const emit = defineEmits<{ select: [id: string]; open: [row: Row]; sort: [field: string] }>()
const visible = computed(() => props.fields.filter(f => !f.hidden))
function display(value: unknown) { return value === null || value === undefined || value === '' ? '—' : String(value) }
</script>

<template>
  <UCard :ui="{ body: 'p-0 overflow-x-auto' }">
    <table class="w-full text-sm">
      <thead class="border-b bg-muted/30">
        <tr><th class="w-10 px-3 py-2" /><th v-for="field in visible" :key="field.id" class="px-3 py-2 text-left font-medium"><button v-if="field.sortable" type="button" class="hover:underline" @click="emit('sort', field.name)">{{ field.label || field.name }}</button><span v-else>{{ field.label || field.name }}</span></th><th class="w-20 px-3 py-2" /></tr>
      </thead>
      <tbody>
        <tr v-for="row in rows" :key="row.id" class="border-b last:border-0 hover:bg-elevated/30">
          <td class="px-3 py-2"><UCheckbox :model-value="selected?.has(row.id)" @update:model-value="emit('select', row.id)" /></td>
          <td v-for="field in visible" :key="field.id" class="px-3 py-2">{{ display(row.payload[field.name]) }}</td>
          <td class="px-3 py-2 text-right"><UButton size="xs" variant="ghost" @click="emit('open', row)">Open</UButton></td>
        </tr>
        <tr v-if="!rows.length"><td :colspan="visible.length + 2" class="px-3 py-10 text-center text-muted">No records</td></tr>
      </tbody>
    </table>
  </UCard>
</template>
