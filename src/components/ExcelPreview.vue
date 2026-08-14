<script setup lang="ts">
import { computed } from 'vue'

import type { ExcelPreview } from '@/types/template'

const props = defineProps<{
  preview: ExcelPreview | null
}>()

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
</script>

<template>
  <div v-if="preview" class="overflow-auto rounded-lg border border-neutral-200">
    <table class="min-w-full border-collapse text-sm">
      <thead class="sticky top-0 bg-[#fafafa]">
        <tr>
          <th class="w-14 border-b border-r border-neutral-200 px-2 py-1.5 text-left text-xs font-medium text-neutral-500">
            行
          </th>
          <th
            v-for="col in columnIndexes"
            :key="col"
            class="min-w-32 border-b border-neutral-200 px-2 py-1.5 text-left text-xs font-medium text-neutral-500"
          >
            列 {{ col + 1 }}
          </th>
        </tr>
      </thead>
      <tbody class="divide-y divide-neutral-200">
        <tr
          v-for="(row, rowOffset) in preview.rows"
          :key="preview.start_row + rowOffset"
          :class="isHeaderRow(rowOffset) ? 'bg-neutral-100' : 'bg-white'"
        >
          <td class="border-r border-neutral-200 px-2 py-1 text-xs text-neutral-500">
            {{ preview.start_row + rowOffset }}
          </td>
          <td
            v-for="col in columnIndexes"
            :key="`${rowOffset}-${col}`"
            class="max-w-56 truncate px-2 py-1 text-neutral-800"
            :class="isHeaderRow(rowOffset) ? 'font-medium' : ''"
            :title="cellValue(row, col)"
          >
            {{ cellValue(row, col) }}
          </td>
        </tr>
      </tbody>
    </table>
  </div>
  <div v-else class="rounded-lg border border-dashed border-neutral-200 px-4 py-8 text-sm text-neutral-500">
    选择 Excel 后将在这里预览表头附近的行。
  </div>
</template>
