<script setup lang="ts">
import type { ImportSession } from '@/types/import'

import { Plus, Trash2 } from '@lucide/vue'
import { storeToRefs } from 'pinia'
import BaseButton from '@/components/ui/BaseButton.vue'
import StatusBadge from '@/components/ui/StatusBadge.vue'
import { useImportStore } from '@/stores/import'

defineProps<{
  sessions: ImportSession[]
  activeId: string | null
  confirmedCount: number
  analyzing: boolean
}>()

const emit = defineEmits<{
  select: [id: string]
  add: []
  remove: [id: string]
}>()

const importStore = useImportStore()
const { readFilteredOnly } = storeToRefs(importStore)

function statusKind(session: ImportSession) {
  if (session.error) {
    return 'error' as const
  }
  return session.confirmed ? ('confirmed' as const) : ('pending' as const)
}

async function onReadFilteredOnlyChange(event: Event) {
  const target = event.target as HTMLInputElement
  await importStore.setReadFilteredOnly(target.checked)
}
</script>

<template>
  <aside class="flex min-h-0 w-64 shrink-0 flex-col overflow-hidden border-l border-primary-200">
    <div class="border-b border-primary-200 px-3 py-3">
      <div class="text-sm font-medium text-primary-950">
        源数据文件
      </div>
      <div class="mt-1 text-xs text-primary-500">
        {{ sessions.length }} 个文件 · {{ confirmedCount }} 已确认
      </div>
      <label class="mt-3 flex cursor-pointer items-start gap-2 text-xs text-primary-700">
        <input
          type="checkbox"
          class="mt-0.5"
          :checked="readFilteredOnly"
          :disabled="analyzing"
          @change="onReadFilteredOnlyChange"
        >
        <span>只读筛选后的数据</span>
      </label>
      <p class="mt-1 text-[11px] leading-4 text-primary-400">
        跳过 Excel 中已隐藏或被筛选隐藏的行；.xls 文件无法识别筛选状态。
      </p>
    </div>
    <div class="min-h-0 flex-1 overflow-auto p-2">
      <div
        v-for="session in sessions"
        :key="session.id"
        class="mb-1 flex items-start"
        :class="session.id === activeId ? 'bg-primary-100' : 'hover:bg-primary-50'"
      >
        <button
          type="button"
          class="flex min-w-0 flex-1 items-start justify-between gap-2 px-2 py-2 text-left text-sm"
          @click="emit('select', session.id)"
        >
          <span class="min-w-0">
            <span class="block truncate text-primary-900">{{ session.file_name }}</span>
            <span class="mt-0.5 block text-xs text-primary-500">表头第 {{ session.header_row }} 行</span>
          </span>
          <StatusBadge :kind="statusKind(session)" />
        </button>
        <BaseButton
          variant="ghost"
          size="sm"
          class="mt-1 shrink-0 text-primary-400 hover:text-error"
          :disabled="analyzing"
          :title="`移除 ${session.file_name}`"
          @click.stop="emit('remove', session.id)"
        >
          <Trash2 class="h-3.5 w-3.5" />
        </BaseButton>
      </div>
      <p v-if="sessions.length === 0" class="px-2 py-6 text-center text-xs text-primary-500">
        拖拽或添加一个或多个数据 Excel，系统会分别检测表头并推荐映射。
      </p>
    </div>
    <div class="border-t border-primary-200 p-2">
      <BaseButton class="w-full" :disabled="analyzing" @click="emit('add')">
        <Plus class="h-3.5 w-3.5" />
        添加文件
      </BaseButton>
    </div>
  </aside>
</template>
