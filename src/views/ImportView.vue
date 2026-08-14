<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { computed } from 'vue'

import ExcelDropZone from '@/components/ExcelDropZone.vue'
import ImportFileList from '@/components/ImportFileList.vue'
import ImportSessionPane from '@/components/ImportSessionPane.vue'
import StepNav from '@/components/StepNav.vue'
import { useExcelFileDrop } from '@/composables/useExcelFileDrop'
import { useImportStore } from '@/stores/import'
import { useTemplateStore } from '@/stores/template'

const emit = defineEmits<{
  prev: []
  next: []
}>()

const importStore = useImportStore()
const templateStore = useTemplateStore()
const {
  sessions,
  activeSessionId,
  activeSession,
  analyzingFiles,
  analyzingLabel,
  confirmedCount,
  allConfirmed,
  mappingError,
  errorMessage,
} = storeToRefs(importStore)
const { currentTemplate } = storeToRefs(templateStore)

const { isOver, isValid, onDragEnter, onDragOver, onDragLeave, onDrop } = useExcelFileDrop({
  onDropMany: paths => importStore.addFilesFromPaths(paths),
  onInvalid: message => importStore.setError(message),
  missingPathMessage: '无法读取文件路径，请点击“添加文件”',
})

const showAddOverlay = computed(() => sessions.value.length > 0 && isOver.value)

const overlayClass = computed(() =>
  isValid.value
    ? 'border-primary-950 bg-white/90 text-primary-950'
    : 'border-error bg-error/10 text-error-fg',
)

const nextDisabled = computed(() => !allConfirmed.value || analyzingFiles.value)
const nextTitle = computed(() => {
  if (analyzingFiles.value) {
    return '正在分析文件…'
  }
  if (sessions.value.length === 0) {
    return '请先添加数据文件'
  }
  if (!allConfirmed.value) {
    return '请先确认每个数据文件的映射'
  }
  return ''
})
</script>

<template>
  <div
    class="relative flex h-full min-h-0 flex-col"
    @dragenter.prevent="onDragEnter"
    @dragover.prevent="onDragOver"
    @dragleave="onDragLeave"
    @drop.prevent="onDrop"
  >
    <div class="flex min-h-0 flex-1">
      <section class="flex min-w-0 flex-1 flex-col overflow-hidden">
        <div v-if="errorMessage" class="border-b border-error/20 bg-error/10 px-5 py-2 text-sm text-error-fg">
          {{ errorMessage }}
        </div>
        <div v-if="analyzingFiles" class="border-b border-primary-200 bg-primary-50 px-5 py-2 text-xs text-primary-500">
          正在分析 {{ analyzingLabel }} …
        </div>

        <ExcelDropZone
          v-if="!activeSession"
          class="min-h-0 flex-1"
          :is-over="isOver"
          :is-valid="isValid"
          :loading="analyzingFiles"
          idle-title="拖拽 Excel 数据文件到此处"
          loading-title="正在分析数据文件…"
          drop-title="松开以添加数据文件"
          idle-hint="可一次添加多个文件，或点击选择"
          @browse="importStore.addFiles"
        />

        <ImportSessionPane
          v-else
          :session="activeSession"
          :template-columns="currentTemplate?.columns ?? []"
          :mapping-error="mappingError"
          @confirm="importStore.confirmActive"
          @sheet-change="importStore.changeSheet"
          @header-row-change="importStore.changeHeaderRow"
          @mapping-change="importStore.changeMapping"
        />
      </section>

      <ImportFileList
        :sessions="sessions"
        :active-id="activeSessionId"
        :confirmed-count="confirmedCount"
        :analyzing="analyzingFiles"
        @select="importStore.selectSession"
        @add="importStore.addFiles"
      />
    </div>

    <StepNav
      :next-disabled="nextDisabled"
      :next-title="nextTitle"
      @prev="emit('prev')"
      @next="emit('next')"
    >
      {{ nextTitle || '全部文件已确认，可以进入汇总' }}
    </StepNav>

    <div
      v-if="showAddOverlay"
      class="pointer-events-none absolute inset-0 z-10 flex items-center justify-center border-2 border-dashed text-sm font-medium backdrop-blur-[2px] transition-colors duration-200 motion-reduce:transition-none"
      :class="overlayClass"
    >
      {{ isValid ? '松开以添加数据文件' : '仅支持 .xlsx / .xls / .xlsm' }}
    </div>
  </div>
</template>
