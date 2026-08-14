<script setup lang="ts">
import { FileSpreadsheet } from '@lucide/vue'
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    isOver: boolean
    isValid: boolean
    loading: boolean
    idleTitle?: string
    loadingTitle?: string
    dropTitle?: string
    invalidTitle?: string
    idleHint?: string
  }>(),
  {
    idleTitle: '拖拽 Excel 文件到此处',
    loadingTitle: '正在读取…',
    dropTitle: '松开以加载文件',
    invalidTitle: '仅支持 .xlsx / .xls / .xlsm',
    idleHint: '或点击选择 .xlsx / .xls / .xlsm 文件',
  },
)

const emit = defineEmits<{
  browse: []
}>()

const zoneClass = computed(() => {
  if (props.isOver && !props.isValid) {
    return 'border-error bg-error/10 text-error-fg'
  }
  if (props.isOver) {
    return 'border-primary-950 bg-primary-50 text-primary-950'
  }
  return 'border-primary-300 bg-primary-50 text-primary-500 hover:border-primary-400 hover:bg-white hover:text-primary-700'
})

const title = computed(() => {
  if (props.loading) {
    return props.loadingTitle
  }
  if (props.isOver && !props.isValid) {
    return props.invalidTitle
  }
  if (props.isOver) {
    return props.dropTitle
  }
  return props.idleTitle
})
</script>

<template>
  <button
    type="button"
    class="flex h-full min-h-48 w-full flex-col items-center justify-center rounded-md border border-dashed px-6 text-center transition-colors duration-200 motion-reduce:transition-none"
    :class="zoneClass"
    :disabled="loading"
    @click="emit('browse')"
  >
    <FileSpreadsheet
      class="h-8 w-8"
      :class="isOver && !isValid ? 'text-error' : isOver ? 'text-primary-800' : 'text-primary-400'"
    />
    <div class="mt-3 text-sm font-medium">
      {{ title }}
    </div>
    <div v-if="!loading" class="mt-1 text-xs">
      {{ isOver ? '也可点击选择文件' : idleHint }}
    </div>
  </button>
</template>
