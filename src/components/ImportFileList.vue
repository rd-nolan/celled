<script setup lang="ts">
import { Plus } from '@lucide/vue'

import BaseButton from '@/components/ui/BaseButton.vue'
import StatusBadge from '@/components/ui/StatusBadge.vue'
import type { ImportSession } from '@/types/import'

defineProps<{
  sessions: ImportSession[]
  activeId: string | null
  confirmedCount: number
  analyzing: boolean
}>()

const emit = defineEmits<{
  select: [id: string]
  add: []
}>()

function statusKind(session: ImportSession) {
  if (session.error) {
    return 'error' as const
  }
  return session.confirmed ? ('confirmed' as const) : ('pending' as const)
}
</script>

<template>
  <aside class="flex w-64 shrink-0 flex-col overflow-hidden rounded-lg border border-neutral-200 bg-white">
    <div class="border-b border-neutral-200 px-3 py-3">
      <div class="text-sm font-medium text-neutral-900">数据文件</div>
      <div class="mt-1 text-xs text-neutral-500">
        {{ sessions.length }} 个文件 · {{ confirmedCount }} 已确认
      </div>
    </div>
    <div class="min-h-0 flex-1 overflow-auto p-2">
      <button
        v-for="session in sessions"
        :key="session.id"
        type="button"
        class="mb-1 flex w-full items-start justify-between gap-2 rounded-md px-2 py-2 text-left text-sm"
        :class="session.id === activeId ? 'bg-neutral-100' : 'hover:bg-[#fafafa]'"
        @click="emit('select', session.id)"
      >
        <span class="min-w-0">
          <span class="block truncate text-neutral-900">{{ session.file_name }}</span>
          <span class="mt-0.5 block text-xs text-neutral-500">表头第 {{ session.header_row }} 行</span>
        </span>
        <StatusBadge :kind="statusKind(session)" />
      </button>
      <p v-if="sessions.length === 0" class="px-2 py-6 text-center text-xs text-neutral-500">
        添加一个或多个数据 Excel，系统会分别检测表头并推荐映射。
      </p>
    </div>
    <div class="border-t border-neutral-200 p-2">
      <BaseButton class="w-full" :disabled="analyzing" @click="emit('add')">
        <Plus class="h-3.5 w-3.5" />
        添加文件
      </BaseButton>
    </div>
  </aside>
</template>
