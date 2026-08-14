<script setup lang="ts">
import { computed, shallowRef, watch } from 'vue'

import AppHeader from '@/components/AppHeader.vue'
import AppSteps from '@/components/AppSteps.vue'
import ExportView from '@/views/ExportView.vue'
import ImportView from '@/views/ImportView.vue'
import TemplateView from '@/views/TemplateView.vue'
import { useTemplateStore } from '@/stores/template'

const currentStep = shallowRef(1)
const templateStore = useTemplateStore()
const templateReady = computed(() => templateStore.templateConfirmed)

watch(templateReady, (ready) => {
  if (!ready && currentStep.value > 1) {
    currentStep.value = 1
  }
})
</script>

<template>
  <div class="flex h-screen flex-col overflow-hidden bg-[#f0f0f0] text-neutral-900">
    <AppHeader />
    <AppSteps
      :current="currentStep"
      :template-ready="templateReady"
      @change="currentStep = $event"
    />
    <main class="min-h-0 flex-1 overflow-hidden">
      <TemplateView v-if="currentStep === 1" />
      <ImportView v-else-if="currentStep === 2" />
      <ExportView v-else />
    </main>
  </div>
</template>
