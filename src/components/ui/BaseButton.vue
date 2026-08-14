<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    variant?: 'primary' | 'secondary' | 'ghost'
    size?: 'sm' | 'md'
    disabled?: boolean
    loading?: boolean
    type?: 'button' | 'submit'
  }>(),
  {
    variant: 'secondary',
    size: 'md',
    type: 'button',
  },
)

const buttonClass = computed(() => {
  const variantClass = {
    primary:
      'border-primary-950 bg-primary-950 text-primary-50 hover:bg-primary-800 disabled:border-primary-300 disabled:bg-primary-300',
    secondary:
      'border-primary-200 bg-white text-primary-900 hover:bg-primary-50 disabled:text-primary-400',
    ghost:
      'border-transparent bg-transparent text-primary-600 hover:bg-primary-100 disabled:text-primary-400',
  }[props.variant]

  const sizeClass = props.size === 'sm' ? 'h-8 px-2.5 text-xs' : 'h-9 px-3 text-sm'

  return [
    'inline-flex items-center justify-center gap-1.5 rounded-md border font-medium transition-colors',
    'disabled:cursor-not-allowed',
    sizeClass,
    variantClass,
  ]
})
</script>

<template>
  <button :type="type" :disabled="disabled || loading" :class="buttonClass">
    <span v-if="loading" class="inline-block h-3 w-3 animate-spin rounded-full border border-current border-t-transparent" />
    <slot />
  </button>
</template>
