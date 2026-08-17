import type { ImportSession, OutputFile } from '@/types/import'
import { acceptHMRUpdate, defineStore } from 'pinia'

import { computed, ref, shallowRef } from 'vue'
import {
  analyzeDataExcel,
  asErrorMessage,
  confirmMapping,
  convertFiles,
  mergedOutputFileName,
  pickExcelFiles,
  pickSavePath,
  removeImportSession,
  updateImportHeaderRow,
  updateImportSheet,
  updateMapping,
} from '@/services/tauri'
import { useTemplateStore } from '@/stores/template'

export const useImportStore = defineStore('import', () => {
  const sessions = ref<ImportSession[]>([])
  const activeSessionId = shallowRef<string | null>(null)
  const analyzingFiles = shallowRef(false)
  const converting = shallowRef(false)
  const errorMessage = shallowRef('')
  const warningMessage = shallowRef('')
  const mappingError = shallowRef('')
  const outputs = ref<OutputFile[]>([])
  const analyzingLabel = shallowRef('')
  const convertSucceeded = shallowRef(false)
  const convertError = shallowRef('')
  const readFilteredOnly = ref(true)

  const activeSession = computed(
    () => sessions.value.find(session => session.id === activeSessionId.value) ?? null,
  )
  const confirmedCount = computed(() => sessions.value.filter(session => session.confirmed).length)
  const allConfirmed = computed(
    () => sessions.value.length > 0 && sessions.value.every(session => session.confirmed),
  )
  const mergedOutput = computed(() => outputs.value[0] ?? null)

  function replaceSession(next: ImportSession) {
    const index = sessions.value.findIndex(session => session.id === next.id)
    if (index >= 0) {
      sessions.value[index] = next
    }
    else {
      sessions.value.push(next)
    }
    activeSessionId.value = next.id
  }

  function setError(message: string) {
    errorMessage.value = message
  }

  function clearConvertResult() {
    convertSucceeded.value = false
    convertError.value = ''
    outputs.value = []
  }

  async function refreshSessionsForReadMode() {
    if (sessions.value.length === 0) {
      return
    }
    analyzingFiles.value = true
    errorMessage.value = ''
    clearConvertResult()
    try {
      const current = [...sessions.value]
      for (const session of current) {
        analyzingLabel.value = session.file_name
        replaceSession(
          await updateImportHeaderRow(session.id, session.header_row, readFilteredOnly.value),
        )
      }
    }
    catch (error) {
      errorMessage.value = asErrorMessage(error)
    }
    finally {
      analyzingFiles.value = false
      analyzingLabel.value = ''
    }
  }

  async function setReadFilteredOnly(value: boolean) {
    if (readFilteredOnly.value === value) {
      return
    }
    readFilteredOnly.value = value
    await refreshSessionsForReadMode()
  }

  function setWarning(message: string) {
    warningMessage.value = message
  }

  function clearWarning() {
    warningMessage.value = ''
  }

  async function addFilesFromPaths(paths: string[]) {
    const template = useTemplateStore().currentTemplate
    if (!template) {
      errorMessage.value = '请先确认模板'
      return
    }
    if (paths.length === 0) {
      return
    }

    const existingPaths = new Set(sessions.value.map(session => session.file_path))
    const duplicates: string[] = []
    const newPaths: string[] = []
    const seenInBatch = new Set<string>()

    for (const path of paths) {
      if (existingPaths.has(path) || seenInBatch.has(path)) {
        duplicates.push(path)
      }
      else {
        newPaths.push(path)
        seenInBatch.add(path)
      }
    }

    if (duplicates.length > 0) {
      const names = duplicates.map(path => path.split(/[\\/]/).pop() ?? path).join('、')
      warningMessage.value = `以下文件已在列表中，已跳过：${names}`
    }
    else {
      warningMessage.value = ''
    }

    if (newPaths.length === 0) {
      return
    }

    analyzingFiles.value = true
    errorMessage.value = ''
    clearConvertResult()
    try {
      for (const path of newPaths) {
        analyzingLabel.value = path.split(/[\\/]/).pop() ?? path
        const session = await analyzeDataExcel(
          path,
          template.id,
          undefined,
          readFilteredOnly.value,
        )
        replaceSession(session)
      }
    }
    catch (error) {
      errorMessage.value = asErrorMessage(error)
    }
    finally {
      analyzingFiles.value = false
      analyzingLabel.value = ''
    }
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
    await addFilesFromPaths(paths)
  }

  function selectSession(id: string) {
    activeSessionId.value = id
    mappingError.value = ''
  }

  async function removeSession(id: string) {
    errorMessage.value = ''
    mappingError.value = ''
    try {
      await removeImportSession(id)
    }
    catch (error) {
      errorMessage.value = asErrorMessage(error)
      return
    }

    const index = sessions.value.findIndex(session => session.id === id)
    if (index < 0) {
      return
    }

    sessions.value.splice(index, 1)
    clearConvertResult()

    if (activeSessionId.value === id) {
      const next = sessions.value[index] ?? sessions.value[index - 1] ?? null
      activeSessionId.value = next?.id ?? null
    }
  }

  async function changeHeaderRow(headerRow: number) {
    const session = activeSession.value
    if (!session) {
      return
    }
    errorMessage.value = ''
    mappingError.value = ''
    try {
      replaceSession(
        await updateImportHeaderRow(session.id, headerRow, readFilteredOnly.value),
      )
    }
    catch (error) {
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
      replaceSession(
        await updateImportSheet(session.id, sheetName, readFilteredOnly.value),
      )
    }
    catch (error) {
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
    }
    catch (error) {
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
    }
    catch (error) {
      mappingError.value = asErrorMessage(error)
    }
  }

  async function startConvert() {
    if (!allConfirmed.value) {
      return
    }
    const defaultName = mergedOutputFileName(useTemplateStore().currentTemplate?.file_name)
    const outputPath = await pickSavePath(defaultName)
    if (!outputPath) {
      return
    }
    converting.value = true
    errorMessage.value = ''
    convertSucceeded.value = false
    convertError.value = ''
    outputs.value = []
    try {
      outputs.value = await convertFiles(
        sessions.value.map(session => session.id),
        outputPath,
      )
      convertSucceeded.value = true
    }
    catch (error) {
      const message = asErrorMessage(error)
      convertError.value = message
      errorMessage.value = message
    }
    finally {
      converting.value = false
    }
  }

  function reset() {
    sessions.value = []
    activeSessionId.value = null
    outputs.value = []
    errorMessage.value = ''
    warningMessage.value = ''
    mappingError.value = ''
    convertSucceeded.value = false
    convertError.value = ''
    readFilteredOnly.value = true
  }

  return {
    sessions,
    activeSessionId,
    activeSession,
    analyzingFiles,
    analyzingLabel,
    converting,
    errorMessage,
    warningMessage,
    mappingError,
    outputs,
    convertSucceeded,
    convertError,
    confirmedCount,
    allConfirmed,
    mergedOutput,
    readFilteredOnly,
    setError,
    setWarning,
    clearWarning,
    setReadFilteredOnly,
    addFilesFromPaths,
    addFiles,
    selectSession,
    removeSession,
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
