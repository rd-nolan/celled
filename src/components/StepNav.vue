<script setup lang="ts">
import BaseButton from '@/components/ui/BaseButton.vue'

withDefaults(
  defineProps<{
    showPrev?: boolean
    prevDisabled?: boolean
    nextDisabled?: boolean
    nextLoading?: boolean
    nextLabel?: string
    nextTitle?: string
    nextVariant?: 'primary' | 'secondary'
  }>(),
  {
    showPrev: true,
    nextLabel: '下一步',
    nextVariant: 'primary',
  },
)

const emit = defineEmits<{
  prev: []
  next: []
}>()
</script>

<template>
  <footer class="flex shrink-0 items-center justify-between gap-4 border-t border-primary-200 px-5 py-3">
    <div class="min-w-0 text-xs text-primary-500">
      <slot />
    </div>
    <div class="flex shrink-0 items-center gap-2">
      <slot name="actions" />
      <BaseButton
        v-if="showPrev"
        :disabled="prevDisabled"
        @click="emit('prev')"
      >
        上一步
      </BaseButton>
      <BaseButton
        :variant="nextVariant"
        :disabled="nextDisabled"
        :loading="nextLoading"
        :title="nextTitle"
        @click="emit('next')"
      >
        {{ nextLabel }}
      </BaseButton>
    </div>
  </footer>
</template>
