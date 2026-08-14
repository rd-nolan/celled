<script setup lang="ts">
import { storeToRefs } from 'pinia'

import BaseButton from '@/components/ui/BaseButton.vue'
import BaseCard from '@/components/ui/BaseCard.vue'
import ExcelPreview from '@/components/ExcelPreview.vue'
import HeaderRowSelector from '@/components/HeaderRowSelector.vue'
import { useTemplateStore } from '@/stores/template'

const templateStore = useTemplateStore()
const { templateAnalysis, loading, errorMessage, canConfirm, templateConfirmed, currentTemplate } =
  storeToRefs(templateStore)

function onSheetChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value
  void templateStore.changeSheet(value)
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col p-4">
    <BaseCard padded>
      <template #header>
        <div class="flex items-center justify-between gap-4">
          <div>
            <div class="text-sm font-medium text-neutral-900">模板文件</div>
            <div class="mt-0.5 text-xs text-neutral-500">
              先确认模板表头，再导入数据 Excel。AI 只推荐映射，不会自动提交。
            </div>
          </div>
          <BaseButton :loading="loading" @click="templateStore.chooseTemplate">
            选择模板
          </BaseButton>
        </div>
      </template>

      <div v-if="errorMessage" class="mb-3 rounded-md bg-red-50 px-3 py-2 text-sm text-red-700">
        {{ errorMessage }}
      </div>

      <div v-if="!templateAnalysis" class="flex h-full items-center justify-center text-sm text-neutral-500">
        选择一个 Excel 模板。系统会扫描前 15 行并检测最可能的表头。
      </div>

      <div v-else class="flex h-full min-h-0 flex-col gap-4">
        <div class="grid grid-cols-3 gap-4 rounded-lg border border-neutral-200 bg-[#fafafa] p-3">
          <div>
            <div class="text-xs text-neutral-500">模板文件</div>
            <div class="mt-1 truncate text-sm text-neutral-900">{{ templateAnalysis.file_name }}</div>
          </div>
          <label class="text-sm text-neutral-700">
            <span class="block text-xs text-neutral-500">Sheet</span>
            <select
              class="mt-1 h-8 w-full rounded-md border border-neutral-200 bg-white px-2 text-sm"
              :value="templateAnalysis.sheet_name"
              @change="onSheetChange"
            >
              <option v-for="sheet in templateAnalysis.sheets" :key="sheet" :value="sheet">
                {{ sheet }}
              </option>
            </select>
          </label>
          <HeaderRowSelector
            :header-row="templateAnalysis.detection.row_index"
            :max-row="Math.max(15, templateAnalysis.preview.start_row + templateAnalysis.preview.rows.length - 1)"
            @change="templateStore.changeHeaderRow"
          />
        </div>

        <div class="min-h-0 flex-1">
          <div class="mb-2 text-xs font-medium text-neutral-500">Excel 预览</div>
          <ExcelPreview :preview="templateAnalysis.preview" />
        </div>
      </div>

      <template #footer>
        <div class="flex items-center justify-between">
          <div class="text-xs text-neutral-500">
            <span v-if="templateConfirmed && currentTemplate">
              已确认 {{ currentTemplate.columns.length }} 个模板字段
            </span>
            <span v-else>确认后才会计算字段 Embedding，并允许导入数据文件。</span>
          </div>
          <BaseButton variant="primary" :disabled="!canConfirm" :loading="loading" @click="templateStore.confirm">
            确认模板
          </BaseButton>
        </div>
      </template>
    </BaseCard>
  </div>
</template>
