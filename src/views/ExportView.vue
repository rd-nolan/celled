<script setup lang="ts">
import { storeToRefs } from 'pinia'

import BaseButton from '@/components/ui/BaseButton.vue'
import StatusBadge from '@/components/ui/StatusBadge.vue'
import { useImportStore } from '@/stores/import'

const importStore = useImportStore()
const { sessions, confirmedCount, allConfirmed, converting, outputs, errorMessage } =
  storeToRefs(importStore)
</script>

<template>
  <div class="flex h-full min-h-0 flex-col p-4">
    <section class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-neutral-200 bg-white">
      <div class="border-b border-neutral-200 px-4 py-3">
        <div class="text-sm font-medium text-neutral-900">导出</div>
        <div class="mt-0.5 text-xs text-neutral-500">
          全部文件确认后，按模板字段顺序生成本地 xlsx。转换在 Rust 中完成。
        </div>
      </div>

      <div class="min-h-0 flex-1 overflow-auto p-4">
        <div v-if="errorMessage" class="mb-3 rounded-md bg-red-50 px-3 py-2 text-sm text-red-700">
          {{ errorMessage }}
        </div>

        <div class="mb-4 text-sm text-neutral-600">
          {{ sessions.length }} 个文件，{{ confirmedCount }} 已确认
        </div>

        <ul class="divide-y divide-neutral-200 rounded-lg border border-neutral-200">
          <li
            v-for="session in sessions"
            :key="session.id"
            class="flex items-center justify-between px-3 py-2 text-sm"
          >
            <span class="truncate text-neutral-900">{{ session.file_name }}</span>
            <StatusBadge :kind="session.confirmed ? 'confirmed' : 'pending'" />
          </li>
          <li v-if="sessions.length === 0" class="px-3 py-8 text-center text-sm text-neutral-500">
            还没有数据文件。
          </li>
        </ul>

        <div v-if="outputs.length > 0" class="mt-6">
          <div class="mb-2 text-xs font-medium text-neutral-500">输出文件</div>
          <ul class="space-y-1 text-sm text-neutral-800">
            <li v-for="file in outputs" :key="file.path" class="truncate">
              {{ file.path }}
            </li>
          </ul>
        </div>
      </div>

      <div class="flex items-center justify-between border-t border-neutral-200 bg-[#fafafa] px-4 py-3">
        <div class="text-xs text-neutral-500">
          {{ allConfirmed ? '可以开始转换' : '请先确认每个数据文件的映射' }}
        </div>
        <BaseButton
          variant="primary"
          :disabled="!allConfirmed"
          :loading="converting"
          @click="importStore.startConvert"
        >
          开始转换
        </BaseButton>
      </div>
    </section>
  </div>
</template>
