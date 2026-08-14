<script setup lang="ts">
import { computed, shallowRef, watch } from 'vue'

import AppHeader from '@/components/AppHeader.vue'
import AppSteps from '@/components/AppSteps.vue'
import { useTemplateStore } from '@/stores/template'
import ExportView from '@/views/ExportView.vue'
import ImportView from '@/views/ImportView.vue'
import TemplateView from '@/views/TemplateView.vue'

const currentStep = shallowRef(1)
const templateStore = useTemplateStore()
const templateReady = computed(() => templateStore.templateConfirmed)

watch(templateReady, (ready) => {
  if (ready) {
    if (currentStep.value === 1) {
      currentStep.value = 2
    }
    return
  }
  if (currentStep.value > 1) {
    currentStep.value = 1
  }
})

function goTo(step: number) {
  currentStep.value = step
}
</script>

<template>
  <div class="flex h-screen flex-col overflow-hidden bg-white text-primary-900">
    <AppHeader />
    <AppSteps
      :current="currentStep"
      :template-ready="templateReady"
      @change="goTo"
    />
    <main class="min-h-0 flex-1 overflow-hidden">
      <TemplateView v-if="currentStep === 1" @next="goTo(2)" />
      <ImportView
        v-else-if="currentStep === 2"
        @prev="goTo(1)"
        @next="goTo(3)"
      />
      <ExportView v-else @prev="goTo(2)" />
    </main>
  </div>
</template>
