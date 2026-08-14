export interface TemplateColumn {
  index: number
  name: string
  normalized_name: string
  path: string | null
}

export interface TemplateSchema {
  id: string
  file_name: string
  file_path: string
  sheet_name: string
  header_start_row: number
  header_end_row: number
  data_start_row: number
  columns: TemplateColumn[]
}

export interface HeaderDetectionResult {
  row_index: number
  confidence: number
  headers: string[]
}

export interface ExcelPreview {
  sheet_name: string
  header_row: number
  start_row: number
  rows: string[][]
}

export interface TemplateAnalysis {
  file_path: string
  file_name: string
  sheets: string[]
  sheet_name: string
  detection: HeaderDetectionResult
  preview: ExcelPreview
}

export interface AppInfo {
  embedding_backend: string
  embedding_model_version: string
}
