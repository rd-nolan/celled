<script setup lang="ts">
defineProps<{
  padded?: boolean
}>()
</script>

<template>
  <section class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden rounded-lg border border-primary-200 bg-white">
    <div v-if="$slots.header" class="shrink-0 border-b border-primary-200 px-4 py-3">
      <slot name="header" />
    </div>
    <div class="min-h-0 flex-1 overflow-auto" :class="padded ? 'p-4' : ''">
      <slot />
    </div>
    <div v-if="$slots.footer" class="shrink-0 border-t border-primary-200 bg-primary-50 px-4 py-3">
      <slot name="footer" />
    </div>
  </section>
</template>
