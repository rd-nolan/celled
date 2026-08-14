<script setup lang="ts">
import type { AppInfo } from '@/types/template'

import { shallowRef } from 'vue'
import BaseButton from '@/components/ui/BaseButton.vue'
import { getAppInfo } from '@/services/tauri'

const aboutOpen = shallowRef(false)
const info = shallowRef<AppInfo | null>(null)

async function openAbout() {
  aboutOpen.value = true
  try {
    info.value = await getAppInfo()
  }
  catch {
    info.value = null
  }
}
</script>

<template>
  <header class="flex h-12 shrink-0 items-center justify-between border-b border-primary-200 px-5">
    <span class="font-display text-[15px] font-semibold tracking-tight text-primary-950">Celled</span>
    <BaseButton variant="ghost" size="sm" @click="openAbout">
      关于
    </BaseButton>
  </header>

  <div
    v-if="aboutOpen"
    class="fixed inset-0 z-20 flex items-center justify-center bg-primary-950/20"
    @click.self="aboutOpen = false"
  >
    <div class="w-[360px] rounded-lg border border-primary-200 bg-white p-4 shadow-sm">
      <div class="font-display text-sm font-medium text-primary-950">
        关于 Celled
      </div>
      <p class="mt-2 text-sm text-primary-600">
        本地 Excel 表头匹配与数据转换工具。文件与字段均在本机处理，不上传网络。
      </p>
      <dl class="mt-3 space-y-1 text-xs text-primary-500">
        <div class="flex justify-between gap-4">
          <dt>Embedding</dt>
          <dd>{{ info?.embedding_backend ?? '—' }}</dd>
        </div>
        <div class="flex justify-between gap-4">
          <dt>模型版本</dt>
          <dd>{{ info?.embedding_model_version ?? '—' }}</dd>
        </div>
      </dl>
      <div class="mt-4 flex justify-end">
        <BaseButton size="sm" @click="aboutOpen = false">
          关闭
        </BaseButton>
      </div>
    </div>
  </div>
</template>
