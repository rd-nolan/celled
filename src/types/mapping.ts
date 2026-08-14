export type MappingSource =
  | 'exact'
  | 'normalized_exact'
  | 'history'
  | 'alias'
  | 'embedding'
  | 'manual'
  | 'unmatched'

export interface MatchCandidate {
  template_column_index: number
  template_header: string
  score: number
}

export interface HeaderMapping {
  source_column_index: number
  source_header: string
  normalized_source_header: string
  target_column_index: number | null
  target_header: string | null
  score: number | null
  source: MappingSource
  candidates: MatchCandidate[]
}

export interface SourceColumn {
  index: number
  header: string
  normalized_header: string
  sample_values: string[]
}
