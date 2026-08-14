<script setup lang="ts">
import { computed } from 'vue'

import StatusBadge from '@/components/ui/StatusBadge.vue'
import type { TemplateColumn } from '@/types/template'
import type { HeaderMapping, MatchCandidate, SourceColumn } from '@/types/mapping'

const props = defineProps<{
  mappings: HeaderMapping[]
  sourceColumns: SourceColumn[]
  templateColumns: TemplateColumn[]
}>()

const emit = defineEmits<{
  change: [sourceColumnIndex: number, targetColumnIndex: number | null]
}>()

function samplesFor(index: number) {
  return props.sourceColumns.find((column) => column.index === index)?.sample_values ?? []
}

function selectedValue(mapping: HeaderMapping) {
  return mapping.target_column_index === null ? '' : String(mapping.target_column_index)
}

function otherOptions(mapping: HeaderMapping) {
  const recommended = new Set(mapping.candidates.map((item) => item.template_column_index))
  return props.templateColumns.filter((column) => !recommended.has(column.index))
}

function onChange(sourceColumnIndex: number, event: Event) {
  const raw = (event.target as HTMLSelectElement).value
  emit('change', sourceColumnIndex, raw === '' ? null : Number(raw))
}

function similarityText(mapping: HeaderMapping) {
  if (mapping.source !== 'embedding' || mapping.score === null) {
    return '—'
  }
  return `${Math.round(mapping.score * 100)}%`
}

function candidateLabel(mapping: HeaderMapping, candidate: MatchCandidate) {
  if (mapping.source === 'embedding') {
    return `${candidate.template_header} ${Math.round(candidate.score * 100)}%`
  }
  return candidate.template_header
}

const hasRows = computed(() => props.mappings.length > 0)
</script>

<template>
  <div class="overflow-auto rounded-lg border border-neutral-200">
    <table class="min-w-full border-collapse text-sm">
      <thead class="sticky top-0 bg-[#fafafa]">
        <tr>
          <th class="border-b border-neutral-200 px-3 py-2 text-left font-medium text-neutral-600">数据字段</th>
          <th class="border-b border-neutral-200 px-3 py-2 text-left font-medium text-neutral-600">数据示例</th>
          <th class="border-b border-neutral-200 px-3 py-2 text-left font-medium text-neutral-600">模板字段</th>
          <th class="border-b border-neutral-200 px-3 py-2 text-left font-medium text-neutral-600">匹配来源</th>
          <th class="w-24 border-b border-neutral-200 px-3 py-2 text-left font-medium text-neutral-600">相似度</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-neutral-200">
        <tr v-for="mapping in mappings" :key="mapping.source_column_index" class="bg-white">
          <td class="px-3 py-2 align-top font-medium text-neutral-900">
            {{ mapping.source_header }}
          </td>
          <td class="px-3 py-2 align-top text-xs text-neutral-500">
            <div v-for="sample in samplesFor(mapping.source_column_index).slice(0, 3)" :key="sample">
              {{ sample }}
            </div>
            <span v-if="samplesFor(mapping.source_column_index).length === 0">—</span>
          </td>
          <td class="px-3 py-2 align-top">
            <select
              class="h-8 w-full min-w-40 rounded-md border border-neutral-200 bg-white px-2 text-sm"
              :value="selectedValue(mapping)"
              @change="onChange(mapping.source_column_index, $event)"
            >
              <option value="">不映射 / 忽略此列</option>
              <optgroup v-if="mapping.candidates.length > 0" label="推荐">
                <option
                  v-for="candidate in mapping.candidates"
                  :key="`c-${candidate.template_column_index}`"
                  :value="candidate.template_column_index"
                >
                  {{ candidateLabel(mapping, candidate) }}
                </option>
              </optgroup>
              <optgroup label="全部模板字段">
                <option
                  v-for="column in otherOptions(mapping)"
                  :key="column.index"
                  :value="column.index"
                >
                  {{ column.name }}
                </option>
              </optgroup>
            </select>
          </td>
          <td class="px-3 py-2 align-top">
            <StatusBadge :kind="mapping.source" />
          </td>
          <td class="px-3 py-2 align-top text-neutral-600">
            {{ similarityText(mapping) }}
          </td>
        </tr>
        <tr v-if="!hasRows">
          <td colspan="5" class="px-3 py-8 text-center text-neutral-500">暂无字段可映射</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
