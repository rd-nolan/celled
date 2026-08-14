<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from 'vue'

import type { ExcelPreview } from '@/types/template'

const props = defineProps<{
  preview: ExcelPreview | null
}>()

const previewEl = useTemplateRef<HTMLElement>('preview')

const columnCount = computed(() => {
  const rows = props.preview?.rows ?? []
  return rows.reduce((max, row) => Math.max(max, row.length), 0)
})

const columnIndexes = computed(() => Array.from({ length: columnCount.value }, (_, i) => i))

function cellValue(row: string[], index: number) {
  return row[index] ?? ''
}

function isHeaderRow(offset: number) {
  if (!props.preview) {
    return false
  }
  return props.preview.start_row + offset === props.preview.header_row
}

watch(
  () => props.preview,
  async () => {
    await nextTick()
    const root = previewEl.value
    const header = root?.querySelector<HTMLElement>('[data-excel-header]')
    if (!root || !header) {
      return
    }
    const thead = root.querySelector('thead')
    const theadHeight = thead?.getBoundingClientRect().height ?? 0
    root.scrollTop = Math.max(0, header.offsetTop - theadHeight)
  },
  { immediate: true },
)
</script>

<template>
  <div v-if="preview" ref="preview" class="excel-preview overflow-auto border border-primary-200">
    <table class="min-w-full border-collapse text-sm">
      <thead class="sticky top-0 bg-primary-50">
        <tr>
          <th class="w-14 border-b border-r border-primary-200 px-2 py-1 text-left text-xs font-medium text-primary-500">
            行
          </th>
          <th
            v-for="col in columnIndexes"
            :key="col"
            class="min-w-32 border-b border-primary-200 px-2 py-1 text-left text-xs font-medium text-primary-500"
          >
            列 {{ col + 1 }}
          </th>
        </tr>
      </thead>
      <tbody class="divide-y divide-primary-200">
        <tr
          v-for="(row, rowOffset) in preview.rows"
          :key="preview.start_row + rowOffset"
          :class="isHeaderRow(rowOffset) ? 'bg-primary-100' : 'bg-white'"
          :data-excel-header="isHeaderRow(rowOffset) ? '' : undefined"
        >
          <td class="border-r border-primary-200 px-2 py-0.5 text-xs text-primary-500">
            {{ preview.start_row + rowOffset }}
          </td>
          <td
            v-for="col in columnIndexes"
            :key="`${rowOffset}-${col}`"
            class="max-w-56 truncate px-2 py-0.5 text-primary-800"
            :class="isHeaderRow(rowOffset) ? 'font-medium' : ''"
            :title="cellValue(row, col)"
          >
            {{ cellValue(row, col) }}
          </td>
        </tr>
      </tbody>
    </table>
  </div>
  <div v-else class="border border-dashed border-primary-200 px-4 py-6 text-sm text-primary-500">
    选择 Excel 后将在这里预览表头附近的行。
  </div>
</template>

<style scoped>
.excel-preview {
  max-height: 9rem;
}
</style>
