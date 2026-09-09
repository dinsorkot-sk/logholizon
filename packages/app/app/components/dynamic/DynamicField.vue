<script setup lang="ts">
import { computed } from 'vue'

type Option = { value: string; label: string }
type Field = {
  id: string; name: string; label?: string; description?: string
  type: string; required?: boolean; readonly?: boolean; hidden?: boolean
  can_edit?: boolean; options?: Option[]; ref_entity?: string | null
}
const props = defineProps<{ field: Field; modelValue: unknown; options?: { id: string; label: string }[]; disabled?: boolean }>()
const emit = defineEmits<{ 'update:modelValue': [value: unknown] }>()
const label = computed(() => props.field.label || props.field.name)
const isDisabled = computed(() => props.disabled || props.field.readonly || props.field.can_edit === false)
const selectItems = computed(() => (props.field.options || []).map(o => ({ label: o.label, value: o.value })))
const refItems = computed(() => (props.options || []).map(o => ({ label: o.label, value: o.id })))
function update(value: unknown) { emit('update:modelValue', value) }
</script>

<template>
  <UFormField v-if="!field.hidden" :label="label" :required="field.required" :description="field.description">
    <USelectMenu v-if="field.type === 'select'" :model-value="modelValue as string | undefined" :items="selectItems" value-key="value" :disabled="isDisabled" class="w-full" @update:model-value="update" />
    <USelectMenu v-else-if="field.type === 'reference'" :model-value="modelValue as string | undefined" :items="refItems" value-key="value" :disabled="isDisabled" class="w-full" @update:model-value="update" />
    <UCheckbox v-else-if="field.type === 'boolean' || field.type === 'checkbox'" :model-value="Boolean(modelValue)" :disabled="isDisabled" @update:model-value="update" />
    <UTextarea v-else-if="field.type === 'long_text' || field.type === 'textarea'" :model-value="String(modelValue ?? '')" :disabled="isDisabled" class="w-full" @update:model-value="update" />
    <UInput v-else-if="field.type === 'date'" :model-value="String(modelValue ?? '')" type="date" :disabled="isDisabled" class="w-full" @update:model-value="update" />
    <UInput v-else-if="field.type === 'datetime'" :model-value="String(modelValue ?? '')" type="datetime-local" :disabled="isDisabled" class="w-full" @update:model-value="update" />
    <UInput v-else-if="field.type === 'time'" :model-value="String(modelValue ?? '')" type="time" :disabled="isDisabled" class="w-full" @update:model-value="update" />
    <UInput v-else-if="['number','decimal','currency','percentage','integer'].includes(field.type)" :model-value="String(modelValue ?? '')" type="number" :disabled="isDisabled" class="w-full" @update:model-value="update" />
    <UInput v-else :model-value="String(modelValue ?? '')" :type="field.type === 'email' ? 'email' : field.type === 'url' ? 'url' : 'text'" :disabled="isDisabled" class="w-full" @update:model-value="update" />
  </UFormField>
</template>

