import type { ImportSession, OutputFile } from '@/types/import'
import type { AppInfo, TemplateAnalysis, TemplateSchema } from '@/types/template'

import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'

export const EXCEL_EXTENSIONS = ['xlsx', 'xls', 'xlsm'] as const

const EXCEL_FILTERS = [{ name: 'Excel', extensions: [...EXCEL_EXTENSIONS] }]

export function isExcelPath(path: string): boolean {
  const base = path.split(/[\\/]/).pop() ?? path
  const dot = base.lastIndexOf('.')
  if (dot <= 0) {
    return false
  }
  const ext = base.slice(dot + 1).toLowerCase()
  return (EXCEL_EXTENSIONS as readonly string[]).includes(ext)
}

export async function pickExcelFile(): Promise<string | null> {
  const selected = await open({
    multiple: false,
    filters: EXCEL_FILTERS,
  })
  if (Array.isArray(selected)) {
    return selected[0] ?? null
  }
  return selected
}

export async function pickExcelFiles(): Promise<string[]> {
  const selected = await open({
    multiple: true,
    filters: EXCEL_FILTERS,
  })
  if (!selected) {
    return []
  }
  return Array.isArray(selected) ? selected : [selected]
}

export async function pickDirectory(): Promise<string | null> {
  const selected = await open({
    directory: true,
    multiple: false,
  })
  if (Array.isArray(selected)) {
    return selected[0] ?? null
  }
  return selected
}

export async function pickSavePath(defaultPath: string): Promise<string | null> {
  return save({
    defaultPath,
    filters: [{ name: 'Excel', extensions: ['xlsx'] }],
  })
}

export function analyzeTemplate(path: string, sheetName?: string) {
  return invoke<TemplateAnalysis>('analyze_template', {
    path,
    sheetName: sheetName ?? null,
  })
}

export function updateTemplateHeaderRow(path: string, sheetName: string, headerRow: number) {
  return invoke<TemplateAnalysis>('update_template_header_row', {
    path,
    sheetName,
    headerRow,
  })
}

export function confirmTemplate(payload: {
  filePath: string
  fileName: string
  sheetName: string
  headerRow: number
  dataStartRow?: number
}) {
  return invoke<TemplateSchema>('confirm_template', {
    request: {
      file_path: payload.filePath,
      file_name: payload.fileName,
      sheet_name: payload.sheetName,
      header_row: payload.headerRow,
      data_start_row: payload.dataStartRow ?? null,
    },
  })
}

export function analyzeDataExcel(
  path: string,
  templateId: string,
  sheetName?: string,
  readFilteredOnly = true,
) {
  return invoke<ImportSession>('analyze_data_excel', {
    path,
    templateId,
    sheetName: sheetName ?? null,
    readFilteredOnly,
  })
}

export function updateImportHeaderRow(
  sessionId: string,
  headerRow: number,
  readFilteredOnly?: boolean,
) {
  return invoke<ImportSession>('update_import_header_row', {
    sessionId,
    headerRow,
    readFilteredOnly: readFilteredOnly ?? null,
  })
}

export function updateImportSheet(
  sessionId: string,
  sheetName: string,
  readFilteredOnly?: boolean,
) {
  return invoke<ImportSession>('update_import_sheet', {
    sessionId,
    sheetName,
    readFilteredOnly: readFilteredOnly ?? null,
  })
}

export function updateMapping(
  sessionId: string,
  sourceColumnIndex: number,
  targetColumnIndex: number | null,
) {
  return invoke<ImportSession>('update_mapping', {
    request: {
      session_id: sessionId,
      source_column_index: sourceColumnIndex,
      target_column_index: targetColumnIndex,
    },
  })
}

export function confirmMapping(sessionId: string) {
  return invoke<ImportSession>('confirm_mapping', {
    request: { session_id: sessionId },
  })
}

export function removeImportSession(sessionId: string) {
  return invoke<void>('remove_import_session', { sessionId })
}

export function convertFiles(sessionIds: string[], outputPath: string) {
  return invoke<OutputFile[]>('convert_files', {
    request: {
      session_ids: sessionIds,
      output_path: outputPath,
    },
  })
}

export function mergedOutputFileName(templateFileName?: string | null): string {
  const base = templateFileName?.replace(/\.[^.]+$/, '').trim()
  if (!base) {
    return 'Celled_合并.xlsx'
  }
  return `${base}_合并.xlsx`
}

export function getAppInfo() {
  return invoke<AppInfo>('get_app_info')
}

export function asErrorMessage(error: unknown): string {
  if (typeof error === 'string') {
    return error
  }
  if (error instanceof Error) {
    return error.message
  }
  return '发生未知错误'
}
