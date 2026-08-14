<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  current: number
  templateReady: boolean
}>()

const emit = defineEmits<{
  change: [step: number]
}>()

const steps = computed(() => [
  { id: 1, label: '上传模板' },
  { id: 2, label: '上传源数据文件', disabled: !props.templateReady },
  { id: 3, label: '汇总数据', disabled: !props.templateReady },
])

function select(step: number, disabled: boolean) {
  if (!disabled) {
    emit('change', step)
  }
}
</script>

<template>
  <nav class="flex h-11 shrink-0 items-center gap-2 border-b border-primary-200 px-5 text-sm">
    <template v-for="(step, index) in steps" :key="step.id">
      <button
        type="button"
        class="rounded-md px-2 py-1"
        :class="
          current === step.id
            ? 'font-medium text-primary-950'
            : step.disabled
              ? 'cursor-not-allowed text-primary-400'
              : 'text-primary-500 hover:text-primary-800'
        "
        :disabled="step.disabled"
        @click="select(step.id, Boolean(step.disabled))"
      >
        Step {{ step.id }} {{ step.label }}
      </button>
      <span v-if="index < steps.length - 1" class="text-primary-300">→</span>
    </template>
  </nav>
</template>
