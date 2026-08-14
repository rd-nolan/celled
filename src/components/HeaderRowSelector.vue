<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  headerRow: number
  maxRow?: number
}>()

const emit = defineEmits<{
  change: [row: number]
}>()

const options = computed(() => {
  const max = Math.max(props.maxRow ?? 15, props.headerRow, 1)
  return Array.from({ length: max }, (_, i) => i + 1)
})

function onChange(event: Event) {
  const value = Number((event.target as HTMLSelectElement).value)
  emit('change', value)
}
</script>

<template>
  <label class="flex items-center gap-2 text-sm text-primary-700">
    <span class="shrink-0 text-primary-500">表头所在行</span>
    <select
      class="h-8 rounded-md border border-primary-200 bg-white px-2 text-sm text-primary-900"
      :value="headerRow"
      @change="onChange"
    >
      <option v-for="row in options" :key="row" :value="row">第 {{ row }} 行</option>
    </select>
  </label>
</template>
