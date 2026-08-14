<script setup lang="ts">
import { storeToRefs } from 'pinia'

import BaseButton from '@/components/ui/BaseButton.vue'
import ExcelPreview from '@/components/ExcelPreview.vue'
import HeaderRowSelector from '@/components/HeaderRowSelector.vue'
import ImportFileList from '@/components/ImportFileList.vue'
import MappingTable from '@/components/MappingTable.vue'
import { useImportStore } from '@/stores/import'
import { useTemplateStore } from '@/stores/template'

const importStore = useImportStore()
const templateStore = useTemplateStore()
const {
  sessions,
  activeSessionId,
  activeSession,
  analyzingFiles,
  analyzingLabel,
  confirmedCount,
  mappingError,
  errorMessage,
} = storeToRefs(importStore)
const { currentTemplate } = storeToRefs(templateStore)

function onSheetChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value
  void importStore.changeSheet(value)
}
</script>

<template>
  <div class="flex h-full min-h-0 gap-4 p-4">
    <ImportFileList
      :sessions="sessions"
      :active-id="activeSessionId"
      :confirmed-count="confirmedCount"
      :analyzing="analyzingFiles"
      @select="importStore.selectSession"
      @add="importStore.addFiles"
    />

    <section class="flex min-w-0 flex-1 flex-col overflow-hidden rounded-lg border border-neutral-200 bg-white">
      <div v-if="errorMessage" class="border-b border-red-100 bg-red-50 px-4 py-2 text-sm text-red-700">
        {{ errorMessage }}
      </div>
      <div v-if="analyzingFiles" class="border-b border-neutral-200 bg-[#fafafa] px-4 py-2 text-xs text-neutral-500">
        正在分析 {{ analyzingLabel }} …
      </div>

      <div v-if="!activeSession" class="flex flex-1 items-center justify-center text-sm text-neutral-500">
        添加数据 Excel。每个文件都会独立检测表头并生成映射，需逐个确认。
      </div>

      <template v-else>
        <div class="shrink-0 border-b border-neutral-200 px-4 py-3">
          <div class="flex items-center justify-between gap-4">
            <div>
              <div class="text-sm font-medium text-neutral-900">{{ activeSession.file_name }}</div>
              <div class="mt-0.5 text-xs text-neutral-500">
                数据起始行：第 {{ activeSession.data_start_row }} 行
              </div>
            </div>
            <BaseButton
              variant="primary"
              :disabled="activeSession.confirmed"
              @click="importStore.confirmActive"
            >
              {{ activeSession.confirmed ? '已确认当前文件映射' : '确认当前文件映射' }}
            </BaseButton>
          </div>
          <div class="mt-3 flex flex-wrap items-center gap-4">
            <label class="flex items-center gap-2 text-sm text-neutral-700">
              <span class="text-neutral-500">Sheet</span>
              <select
                class="h-8 rounded-md border border-neutral-200 bg-white px-2 text-sm"
                :value="activeSession.sheet_name"
                @change="onSheetChange"
              >
                <option v-for="sheet in activeSession.sheets" :key="sheet" :value="sheet">
                  {{ sheet }}
                </option>
              </select>
            </label>
            <HeaderRowSelector
              :header-row="activeSession.header_row"
              :max-row="Math.max(15, activeSession.preview.start_row + activeSession.preview.rows.length - 1)"
              @change="importStore.changeHeaderRow"
            />
          </div>
        </div>

        <div class="min-h-0 flex-1 overflow-auto p-4">
          <div class="mb-2 text-xs font-medium text-neutral-500">Excel 预览</div>
          <ExcelPreview :preview="activeSession.preview" />

          <div class="mt-4 mb-2 text-xs font-medium text-neutral-500">字段映射</div>
          <p v-if="mappingError" class="mb-2 rounded-md bg-red-50 px-3 py-2 text-sm text-red-700">
            {{ mappingError }}
          </p>
          <MappingTable
            :mappings="activeSession.mappings"
            :source-columns="activeSession.source_columns"
            :template-columns="currentTemplate?.columns ?? []"
            @change="importStore.changeMapping"
          />
        </div>
      </template>
    </section>
  </div>
</template>
