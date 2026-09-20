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
      <thead class="border-b border-default bg-muted/20">
        <tr>
          <th class="w-10 px-3 py-2.5" />
          <th
            v-for="field in visible"
            :key="field.id"
            class="px-3 py-2.5 text-left font-medium text-muted"
          >
            <button
              v-if="field.sortable"
              type="button"
              class="inline-flex items-center gap-1 text-default hover:text-primary"
              @click="emit('sort', field.name)"
            >
              {{ field.label || field.name }}
              <UIcon name="i-lucide-arrow-up-down" class="h-3 w-3 opacity-50" />
            </button>
            <span v-else>{{ field.label || field.name }}</span>
          </th>
          <th class="w-20 px-3 py-2.5" />
        </tr>
      </thead>
      <tbody class="divide-y divide-default">
        <tr
          v-for="row in rows"
          :key="row.id"
          class="transition-colors hover:bg-elevated/40"
        >
          <td class="px-3 py-2.5">
            <UCheckbox
              :model-value="selected?.has(row.id)"
              @update:model-value="emit('select', row.id)"
            />
          </td>
          <td
            v-for="field in visible"
            :key="field.id"
            class="max-w-[200px] truncate px-3 py-2.5"
          >
            <span :class="display(row.payload[field.name]) === '—' ? 'text-muted' : ''">
              {{ display(row.payload[field.name]) }}
            </span>
          </td>
          <td class="px-3 py-2.5 text-right">
            <UButton size="xs" variant="ghost" @click="emit('open', row)">Open</UButton>
          </td>
        </tr>
        <tr v-if="!rows.length">
          <td :colspan="visible.length + 2" class="px-3 py-12 text-center text-muted">
            <div class="flex flex-col items-center gap-2">
              <UIcon name="i-lucide-inbox" class="h-6 w-6 text-muted/50" />
              <span>No records found</span>
            </div>
          </td>
        </tr>
      </tbody>
    </table>
  </UCard>
</template>
