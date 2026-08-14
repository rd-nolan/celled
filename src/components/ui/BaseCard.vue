<script setup lang="ts">
defineProps<{
  padded?: boolean
}>()
</script>

<template>
  <section class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden rounded-lg border border-neutral-200 bg-white">
    <div v-if="$slots.header" class="shrink-0 border-b border-neutral-200 px-4 py-3">
      <slot name="header" />
    </div>
    <div class="min-h-0 flex-1 overflow-auto" :class="padded ? 'p-4' : ''">
      <slot />
    </div>
    <div v-if="$slots.footer" class="shrink-0 border-t border-neutral-200 bg-[#fafafa] px-4 py-3">
      <slot name="footer" />
    </div>
  </section>
</template>
