<script setup lang="ts">
import DynamicField from './DynamicField.vue'

type Field = { id: string; name: string; type: string; hidden?: boolean; is_status?: boolean; ref_entity?: string | null; [key: string]: unknown }
type Section = { id: string; label: string; fields: string[] }
const props = defineProps<{ fields: Field[]; modelValue: Record<string, unknown>; sections?: Section[] | null; referenceOptions?: Record<string, { id: string; label: string }[]>; disabled?: boolean; errors?: Record<string, string> }>()
const emit = defineEmits<{ 'update:modelValue': [value: Record<string, unknown>] }>()
const visibleFields = computed(() => props.fields.filter(f => !f.hidden && !f.is_status))
const byId = computed(() => new Map<string, Field>(visibleFields.value.map(f => [f.id, f])))
const groups = computed(() => {
  if (!props.sections?.length) return [{ id: 'default', label: '', fields: visibleFields.value }]
  const result = props.sections.map(s => ({ id: s.id, label: s.label, fields: s.fields.map(id => byId.value.get(String(id))).filter((f): f is Field => !!f) }))
  const assigned = new Set(result.flatMap(s => s.fields.map(f => f.id)))
  const other = visibleFields.value.filter(f => !assigned.has(f.id))
  if (other.length) result.push({ id: 'other', label: 'Other', fields: other })
  return result.filter(s => s.fields.length)
})
function setValue(name: string, value: unknown) { emit('update:modelValue', { ...props.modelValue, [name]: value }) }
</script>

<template>
  <div class="space-y-6">
    <section v-for="group in groups" :key="group.id" class="space-y-3">
      <h3 v-if="group.label" class="text-xs font-semibold uppercase tracking-wide text-muted">{{ group.label }}</h3>
      <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
        <DynamicField v-for="field in group.fields" :key="field.id" :field="field" :model-value="modelValue[field.name]" :options="field.ref_entity ? referenceOptions?.[field.ref_entity] : undefined" :disabled="disabled" @update:model-value="setValue(field.name, $event)" />
      </div>
    </section>
  </div>
</template>
