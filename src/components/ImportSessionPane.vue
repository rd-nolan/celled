<script setup lang="ts">
import type { ImportSession } from '@/types/import'
import type { TemplateColumn } from '@/types/template'
import ExcelPreview from '@/components/ExcelPreview.vue'
import HeaderRowSelector from '@/components/HeaderRowSelector.vue'
import MappingTable from '@/components/MappingTable.vue'
import BaseButton from '@/components/ui/BaseButton.vue'

defineProps<{
  session: ImportSession
  templateColumns: TemplateColumn[]
  mappingError: string
}>()

const emit = defineEmits<{
  confirm: []
  sheetChange: [sheetName: string]
  headerRowChange: [row: number]
  mappingChange: [sourceColumnIndex: number, targetColumnIndex: number | null]
}>()

function onSheetChange(event: Event) {
  emit('sheetChange', (event.target as HTMLSelectElement).value)
}
</script>

<template>
  <div class="flex min-h-0 min-w-0 flex-1 flex-col">
    <div class="shrink-0 border-b border-primary-200 px-5 py-3">
      <div class="flex items-center justify-between gap-4">
        <div class="min-w-0">
          <div class="truncate text-sm font-medium text-primary-950">
            {{ session.file_name }}
          </div>
          <div class="mt-0.5 text-xs text-primary-500">
            数据起始行：第 {{ session.data_start_row }} 行
          </div>
        </div>
        <BaseButton
          variant="primary"
          :disabled="session.confirmed"
          @click="emit('confirm')"
        >
          {{ session.confirmed ? '已确认当前文件映射' : '确认当前文件映射' }}
        </BaseButton>
      </div>
      <div class="mt-3 flex flex-wrap items-center gap-4">
        <label class="flex items-center gap-2 text-sm text-primary-700">
          <span class="text-primary-500">Sheet</span>
          <select
            class="h-8 rounded-md border border-primary-200 bg-white px-2 text-sm"
            :value="session.sheet_name"
            @change="onSheetChange"
          >
            <option v-for="sheet in session.sheets" :key="sheet" :value="sheet">
              {{ sheet }}
            </option>
          </select>
        </label>
        <HeaderRowSelector
          :header-row="session.header_row"
          :max-row="Math.max(15, session.preview.start_row + session.preview.rows.length - 1)"
          @change="(row) => emit('headerRowChange', row)"
        />
      </div>
    </div>

    <div class="min-h-0 flex-1 overflow-auto px-5 py-4">
      <div class="mb-2 text-xs font-medium text-primary-500">
        Excel 预览
      </div>
      <ExcelPreview :preview="session.preview" />

      <div class="mt-4 mb-2 text-xs font-medium text-primary-500">
        字段映射
      </div>
      <p v-if="mappingError" class="mb-2 bg-error/10 px-3 py-2 text-sm text-error-fg">
        {{ mappingError }}
      </p>
      <MappingTable
        :mappings="session.mappings"
        :source-columns="session.source_columns"
        :template-columns="templateColumns"
        @change="(source, target) => emit('mappingChange', source, target)"
      />
    </div>
  </div>
</template>
