<script setup lang="ts">
import type { ImportStatus } from '@/types/import'

import type { MappingSource } from '@/types/mapping'
import { computed } from 'vue'

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
    content: '数据推断',
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
    exact: 'bg-success/10 text-success-fg',
    normalized_exact: 'bg-success/10 text-success-fg',
    history: 'bg-accent-100 text-accent-800',
    alias: 'bg-secondary-100 text-secondary-800',
    embedding: 'bg-warning/10 text-warning-fg',
    content: 'bg-accent-100 text-accent-700',
    manual: 'bg-primary-100 text-primary-700',
    unmatched: 'bg-primary-100 text-primary-500',
    pending: 'bg-warning/10 text-warning-fg',
    confirmed: 'bg-success/10 text-success-fg',
    error: 'bg-error/10 text-error-fg',
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
