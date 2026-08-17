import type { HeaderMapping, SourceColumn } from './mapping'
import type { ExcelPreview } from './template'

export type ImportStatus = 'pending' | 'confirmed' | 'error'

export interface ImportSession {
  id: string
  file_path: string
  file_name: string
  sheet_name: string
  sheets: string[]
  header_row: number
  data_start_row: number
  source_columns: SourceColumn[]
  mappings: HeaderMapping[]
  preview: ExcelPreview
  confirmed: boolean
  status: ImportStatus
  error: string | null
  read_filtered_only: boolean
}

export interface OutputFile {
  session_id: string
  path: string
  file_name: string
}
