<script setup lang="ts">
import { CircleAlert, CircleCheck } from '@lucide/vue'
import { storeToRefs } from 'pinia'
import { computed } from 'vue'

import StepNav from '@/components/StepNav.vue'
import StatusBadge from '@/components/ui/StatusBadge.vue'
import { useImportStore } from '@/stores/import'

const emit = defineEmits<{
  prev: []
}>()

const importStore = useImportStore()
const {
  sessions,
  confirmedCount,
  allConfirmed,
  converting,
  mergedOutput,
  errorMessage,
  convertSucceeded,
  convertError,
} = storeToRefs(importStore)

const convertLabel = computed(() => (convertSucceeded.value ? '再次转换' : '开始转换'))
const nextDisabled = computed(() => !allConfirmed.value || converting.value)
const nextTitle = computed(() => {
  if (allConfirmed.value) {
    return ''
  }
  if (sessions.value.length === 0) {
    return '请先添加并确认数据文件'
  }
  return '请先确认每个数据文件的映射'
})
const convertHint = computed(() => {
  if (!allConfirmed.value) {
    return nextTitle.value
  }
  if (convertSucceeded.value) {
    return '可再次选择保存位置并转换'
  }
  return '将按模板表头合并为一个 Excel 文件'
})
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <div class="min-h-0 flex-1 overflow-auto px-5 py-4">
      <div v-if="convertSucceeded" class="mb-4 flex items-start gap-3 bg-success/10 px-3 py-3">
        <CircleCheck class="mt-0.5 h-5 w-5 shrink-0 text-success" />
        <div>
          <div class="text-sm font-medium text-success-fg">
            转换完成
          </div>
          <p class="mt-0.5 text-xs text-primary-600">
            已按模板表头，将所有确认文件的数据合并到一个 Excel 文件。来源列已按源文件名填充。
          </p>
        </div>
      </div>

      <div v-else-if="convertError" class="mb-4 flex items-start gap-3 bg-error/10 px-3 py-3">
        <CircleAlert class="mt-0.5 h-5 w-5 shrink-0 text-error" />
        <div>
          <div class="text-sm font-medium text-error-fg">
            转换失败
          </div>
          <p class="mt-0.5 text-sm text-error-fg">
            {{ convertError }}
          </p>
        </div>
      </div>

      <div v-else-if="errorMessage" class="mb-4 bg-error/10 px-3 py-2 text-sm text-error-fg">
        {{ errorMessage }}
      </div>

      <div class="mb-3 text-sm text-primary-600">
        {{ sessions.length }} 个文件，{{ confirmedCount }} 已确认
      </div>

      <ul class="divide-y divide-primary-200 border border-primary-200">
        <li
          v-for="session in sessions"
          :key="session.id"
          class="flex items-center justify-between px-3 py-2 text-sm"
        >
          <span class="truncate text-primary-900">{{ session.file_name }}</span>
          <StatusBadge :kind="session.confirmed ? 'confirmed' : 'pending'" />
        </li>
        <li v-if="sessions.length === 0" class="px-3 py-8 text-center text-sm text-primary-500">
          还没有数据文件。
        </li>
      </ul>

      <div v-if="mergedOutput" class="mt-6">
        <div class="mb-2 text-xs font-medium text-primary-500">
          输出文件
        </div>
        <p class="truncate text-sm text-primary-800" :title="mergedOutput.path">
          {{ mergedOutput.path }}
        </p>
      </div>
    </div>

    <StepNav
      :next-disabled="nextDisabled"
      :next-loading="converting"
      :next-label="convertLabel"
      :next-title="nextTitle"
      @prev="emit('prev')"
      @next="importStore.startConvert"
    >
      {{ convertHint }}
    </StepNav>
  </div>
</template>
