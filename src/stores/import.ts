import { acceptHMRUpdate, defineStore } from 'pinia'
import { computed, ref, shallowRef } from 'vue'

import {
  analyzeDataExcel,
  asErrorMessage,
  confirmMapping,
  convertFiles,
  pickDirectory,
  pickExcelFiles,
  updateImportHeaderRow,
  updateImportSheet,
  updateMapping,
} from '@/services/tauri'
import type { ImportSession, OutputFile } from '@/types/import'
import { useTemplateStore } from '@/stores/template'

export const useImportStore = defineStore('import', () => {
  const sessions = ref<ImportSession[]>([])
  const activeSessionId = shallowRef<string | null>(null)
  const analyzingFiles = shallowRef(false)
  const converting = shallowRef(false)
  const errorMessage = shallowRef('')
  const mappingError = shallowRef('')
  const outputs = ref<OutputFile[]>([])
  const analyzingLabel = shallowRef('')

  const activeSession = computed(
    () => sessions.value.find((session) => session.id === activeSessionId.value) ?? null,
  )
  const confirmedCount = computed(() => sessions.value.filter((session) => session.confirmed).length)
  const allConfirmed = computed(
    () => sessions.value.length > 0 && sessions.value.every((session) => session.confirmed),
  )

  function replaceSession(next: ImportSession) {
    const index = sessions.value.findIndex((session) => session.id === next.id)
    if (index >= 0) {
      sessions.value[index] = next
    } else {
      sessions.value.push(next)
    }
    activeSessionId.value = next.id
  }

  async function addFiles() {
    const template = useTemplateStore().currentTemplate
    if (!template) {
      errorMessage.value = '请先确认模板'
      return
    }
    const paths = await pickExcelFiles()
    if (paths.length === 0) {
      return
    }
    analyzingFiles.value = true
    errorMessage.value = ''
    try {
      for (const path of paths) {
        analyzingLabel.value = path.split(/[\\/]/).pop() ?? path
        const session = await analyzeDataExcel(path, template.id)
        replaceSession(session)
      }
    } catch (error) {
      errorMessage.value = asErrorMessage(error)
    } finally {
      analyzingFiles.value = false
      analyzingLabel.value = ''
    }
  }

  function selectSession(id: string) {
    activeSessionId.value = id
    mappingError.value = ''
  }

  async function changeHeaderRow(headerRow: number) {
    const session = activeSession.value
    if (!session) {
      return
    }
    errorMessage.value = ''
    mappingError.value = ''
    try {
      replaceSession(await updateImportHeaderRow(session.id, headerRow))
    } catch (error) {
      errorMessage.value = asErrorMessage(error)
    }
  }

  async function changeSheet(sheetName: string) {
    const session = activeSession.value
    if (!session) {
      return
    }
    errorMessage.value = ''
    mappingError.value = ''
    try {
      replaceSession(await updateImportSheet(session.id, sheetName))
    } catch (error) {
      errorMessage.value = asErrorMessage(error)
    }
  }

  async function changeMapping(sourceColumnIndex: number, targetColumnIndex: number | null) {
    const session = activeSession.value
    if (!session) {
      return
    }
    mappingError.value = ''
    try {
      replaceSession(await updateMapping(session.id, sourceColumnIndex, targetColumnIndex))
    } catch (error) {
      mappingError.value = asErrorMessage(error)
    }
  }

  async function confirmActive() {
    const session = activeSession.value
    if (!session) {
      return
    }
    mappingError.value = ''
    try {
      replaceSession(await confirmMapping(session.id))
    } catch (error) {
      mappingError.value = asErrorMessage(error)
    }
  }

  async function startConvert() {
    if (!allConfirmed.value) {
      return
    }
    const outputDir = await pickDirectory()
    if (!outputDir) {
      return
    }
    converting.value = true
    errorMessage.value = ''
    try {
      outputs.value = await convertFiles(
        sessions.value.map((session) => session.id),
        outputDir,
      )
    } catch (error) {
      errorMessage.value = asErrorMessage(error)
    } finally {
      converting.value = false
    }
  }

  function reset() {
    sessions.value = []
    activeSessionId.value = null
    outputs.value = []
    errorMessage.value = ''
    mappingError.value = ''
  }

  return {
    sessions,
    activeSessionId,
    activeSession,
    analyzingFiles,
    analyzingLabel,
    converting,
    errorMessage,
    mappingError,
    outputs,
    confirmedCount,
    allConfirmed,
    addFiles,
    selectSession,
    changeHeaderRow,
    changeSheet,
    changeMapping,
    confirmActive,
    startConvert,
    reset,
  }
})

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useImportStore, import.meta.hot))
}
