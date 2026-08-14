<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { computed } from 'vue'

import ExcelDropZone from '@/components/ExcelDropZone.vue'
import ExcelPreview from '@/components/ExcelPreview.vue'
import HeaderRowSelector from '@/components/HeaderRowSelector.vue'
import StepNav from '@/components/StepNav.vue'
import BaseButton from '@/components/ui/BaseButton.vue'
import { useExcelFileDrop } from '@/composables/useExcelFileDrop'
import { useTemplateStore } from '@/stores/template'

const emit = defineEmits<{
  next: []
}>()

const templateStore = useTemplateStore()
const { templateAnalysis, loading, errorMessage, canConfirm, templateConfirmed, currentTemplate }
  = storeToRefs(templateStore)

const { isOver, isValid, onDragEnter, onDragOver, onDragLeave, onDrop } = useExcelFileDrop({
  onDrop: path => templateStore.loadTemplate(path),
  onInvalid: message => templateStore.setError(message),
  missingPathMessage: '无法读取文件路径，请点击“选择模板”',
})

const showReplaceOverlay = computed(() => Boolean(templateAnalysis.value) && isOver.value)

const overlayClass = computed(() =>
  isValid.value
    ? 'border-primary-950 bg-white/90 text-primary-950'
    : 'border-error bg-error/10 text-error-fg',
)

const nextDisabled = computed(() => !templateConfirmed.value)
const nextTitle = computed(() => {
  if (templateConfirmed.value) {
    return ''
  }
  return templateAnalysis.value ? '请先确认模板' : '请先上传模板'
})

function onSheetChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value
  void templateStore.changeSheet(value)
}

async function onConfirm() {
  await templateStore.confirm()
}
</script>

<template>
  <div
    class="relative flex h-full min-h-0 flex-col"
    @dragenter.prevent="onDragEnter"
    @dragover.prevent="onDragOver"
    @dragleave="onDragLeave"
    @drop.prevent="onDrop"
  >
    <div
      v-if="errorMessage"
      class="shrink-0 border-b border-error/20 bg-error/10 px-5 py-2 text-sm text-error-fg"
    >
      {{ errorMessage }}
    </div>

    <ExcelDropZone
      v-if="!templateAnalysis"
      class="min-h-0 flex-1"
      :is-over="isOver"
      :is-valid="isValid"
      :loading="loading"
      idle-title="拖拽 Excel 模板到此处"
      loading-title="正在读取模板…"
      drop-title="松开以加载模板"
      @browse="templateStore.chooseTemplate"
    />

    <div v-else class="flex min-h-0 flex-1 flex-col gap-4 overflow-auto px-5 py-4">
      <div class="flex flex-wrap items-end gap-x-6 gap-y-3 border-b border-primary-200 pb-4">
        <div class="min-w-0 flex-1">
          <div class="text-xs text-primary-500">
            模板文件
          </div>
          <div class="mt-1 truncate text-sm text-primary-900">
            {{ templateAnalysis.file_name }}
          </div>
        </div>
        <label class="min-w-40 text-sm text-primary-700">
          <span class="block text-xs text-primary-500">Sheet</span>
          <select
            class="mt-1 h-8 w-full rounded-md border border-primary-200 bg-white px-2 text-sm"
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
        <BaseButton :loading="loading" @click="templateStore.chooseTemplate">
          选择模板
        </BaseButton>
      </div>

      <div>
        <div class="mb-2 text-xs font-medium text-primary-500">
          Excel 预览
        </div>
        <ExcelPreview :preview="templateAnalysis.preview" />
      </div>
    </div>

    <StepNav
      :show-prev="false"
      :next-disabled="nextDisabled"
      next-variant="secondary"
      :next-title="nextTitle"
      @next="emit('next')"
    >
      <p v-if="errorMessage" class="text-error-fg">
        {{ errorMessage }}
      </p>
      <span v-else-if="templateConfirmed && currentTemplate">
        已确认 {{ currentTemplate.columns.length }} 个模板字段
      </span>
      <span v-else>确认后才会计算字段 Embedding，并允许导入数据文件。</span>
      <template #actions>
        <BaseButton
          variant="primary"
          :disabled="!canConfirm"
          :loading="loading"
          @click="onConfirm"
        >
          确认模板
        </BaseButton>
      </template>
    </StepNav>

    <div
      v-if="showReplaceOverlay"
      class="pointer-events-none absolute inset-0 z-10 flex items-center justify-center border-2 border-dashed text-sm font-medium backdrop-blur-[2px] transition-colors duration-200 motion-reduce:transition-none"
      :class="overlayClass"
    >
      {{ isValid ? '松开以替换当前模板' : '仅支持 .xlsx / .xls / .xlsm' }}
    </div>
  </div>
</template>
