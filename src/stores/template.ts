import { acceptHMRUpdate, defineStore } from 'pinia'
import { computed, ref, shallowRef } from 'vue'

import {
  analyzeTemplate,
  asErrorMessage,
  confirmTemplate,
  pickExcelFile,
  updateTemplateHeaderRow,
} from '@/services/tauri'
import type { TemplateAnalysis, TemplateSchema } from '@/types/template'

export const useTemplateStore = defineStore('template', () => {
  const currentTemplate = ref<TemplateSchema | null>(null)
  const templateAnalysis = ref<TemplateAnalysis | null>(null)
  const templateConfirmed = shallowRef(false)
  const loading = shallowRef(false)
  const errorMessage = shallowRef('')

  const headerRow = computed(() => templateAnalysis.value?.detection.row_index ?? 1)
  const canConfirm = computed(() => Boolean(templateAnalysis.value) && !loading.value)

  async function chooseTemplate() {
    const path = await pickExcelFile()
    if (!path) {
      return
    }
    await loadTemplate(path)
  }

  async function loadTemplate(path: string, sheetName?: string) {
    loading.value = true
    errorMessage.value = ''
    templateConfirmed.value = false
    currentTemplate.value = null
    const { useImportStore } = await import('@/stores/import')
    useImportStore().reset()
    try {
      templateAnalysis.value = await analyzeTemplate(path, sheetName)
    } catch (error) {
      templateAnalysis.value = null
      errorMessage.value = asErrorMessage(error)
    } finally {
      loading.value = false
    }
  }

  async function changeSheet(sheetName: string) {
    const analysis = templateAnalysis.value
    if (!analysis) {
      return
    }
    await loadTemplate(analysis.file_path, sheetName)
  }

  async function changeHeaderRow(headerRowValue: number) {
    const analysis = templateAnalysis.value
    if (!analysis) {
      return
    }
    loading.value = true
    errorMessage.value = ''
    templateConfirmed.value = false
    currentTemplate.value = null
    const { useImportStore } = await import('@/stores/import')
    useImportStore().reset()
    try {
      templateAnalysis.value = await updateTemplateHeaderRow(
        analysis.file_path,
        analysis.sheet_name,
        headerRowValue,
      )
    } catch (error) {
      errorMessage.value = asErrorMessage(error)
    } finally {
      loading.value = false
    }
  }

  async function confirm() {
    const analysis = templateAnalysis.value
    if (!analysis) {
      return
    }
    loading.value = true
    errorMessage.value = ''
    try {
      const { useImportStore } = await import('@/stores/import')
      useImportStore().reset()
      currentTemplate.value = await confirmTemplate({
        filePath: analysis.file_path,
        fileName: analysis.file_name,
        sheetName: analysis.sheet_name,
        headerRow: analysis.detection.row_index,
      })
      templateConfirmed.value = true
    } catch (error) {
      templateConfirmed.value = false
      errorMessage.value = asErrorMessage(error)
    } finally {
      loading.value = false
    }
  }

  function reset() {
    currentTemplate.value = null
    templateAnalysis.value = null
    templateConfirmed.value = false
    errorMessage.value = ''
  }

  return {
    currentTemplate,
    templateAnalysis,
    templateConfirmed,
    loading,
    errorMessage,
    headerRow,
    canConfirm,
    chooseTemplate,
    loadTemplate,
    changeSheet,
    changeHeaderRow,
    confirm,
    reset,
  }
})

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useTemplateStore, import.meta.hot))
}
