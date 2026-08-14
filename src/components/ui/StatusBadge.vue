<script setup lang="ts">
import { computed } from 'vue'

import type { ImportStatus } from '@/types/import'
import type { MappingSource } from '@/types/mapping'

type BadgeKind = MappingSource | ImportStatus

const props = defineProps<{
  kind: BadgeKind
}>()

const label = computed(() => {
  const labels: Record<BadgeKind, string> = {
    exact: '精确匹配',
    normalized_exact: '精确匹配',
    history: '历史匹配',
    alias: '别名匹配',
    embedding: 'AI 推荐',
    manual: '手动',
    unmatched: '未匹配',
    pending: '待确认',
    confirmed: '已确认',
    error: '错误',
  }
  return labels[props.kind]
})

const badgeClass = computed(() => {
  const classes: Record<BadgeKind, string> = {
    exact: 'bg-emerald-50 text-emerald-800',
    normalized_exact: 'bg-emerald-50 text-emerald-800',
    history: 'bg-sky-50 text-sky-800',
    alias: 'bg-violet-50 text-violet-800',
    embedding: 'bg-amber-50 text-amber-800',
    manual: 'bg-neutral-100 text-neutral-700',
    unmatched: 'bg-neutral-100 text-neutral-500',
    pending: 'bg-amber-50 text-amber-800',
    confirmed: 'bg-emerald-50 text-emerald-800',
    error: 'bg-red-50 text-red-700',
  }
  return classes[props.kind]
})
</script>

<template>
  <span
    class="inline-flex items-center rounded-md px-2 py-0.5 text-xs font-medium"
    :class="badgeClass"
  >
    {{ label }}
  </span>
</template>
