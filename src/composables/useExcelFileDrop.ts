import type { UnlistenFn } from '@tauri-apps/api/event'
import { isTauri } from '@tauri-apps/api/core'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { onMounted, onUnmounted, shallowRef } from 'vue'

import { isExcelPath } from '@/services/tauri'

const INVALID_TYPE_MESSAGE = '仅支持 Excel 文件（.xlsx / .xls / .xlsm）'
const DEFAULT_MISSING_PATH_MESSAGE = '无法读取文件路径，请点击选择文件'
const DROP_DEDUP_MS = 500

interface FileWithPath extends File {
  path?: string
}

export function useExcelFileDrop(options: {
  onDrop?: (path: string) => void | Promise<void>
  onDropMany?: (paths: string[]) => void | Promise<void>
  onInvalid?: (message: string) => void
  missingPathMessage?: string
}) {
  const isOver = shallowRef(false)
  const isValid = shallowRef(false)
  let dropping = false
  let lastDropKey = ''
  let lastDropAt = 0

  function resetHover() {
    isOver.value = false
    isValid.value = false
  }

  function applyHover(valid: boolean) {
    isOver.value = true
    isValid.value = valid
  }

  function emitPaths(paths: string[]) {
    if (dropping) {
      return
    }
    const excelPaths = paths.filter(isExcelPath)
    if (excelPaths.length === 0) {
      options.onInvalid?.(INVALID_TYPE_MESSAGE)
      return
    }
    const key = excelPaths.join('|')
    const now = Date.now()
    if (key === lastDropKey && now - lastDropAt < DROP_DEDUP_MS) {
      return
    }
    lastDropKey = key
    lastDropAt = now
    dropping = true
    const task = options.onDropMany
      ? options.onDropMany(excelPaths)
      : options.onDrop?.(excelPaths[0]!)
    void Promise.resolve(task).finally(() => {
      dropping = false
    })
  }

  function pathsFromDataTransfer(data: DataTransfer | null): string[] {
    if (!data) {
      return []
    }
    return Array.from(data.files)
      .map(file => (file as FileWithPath).path ?? '')
      .filter(Boolean)
  }

  function namesLookLikeExcel(data: DataTransfer | null): boolean {
    if (!data) {
      return true
    }
    const files = Array.from(data.files)
    if (files.length > 0) {
      return files.some(file => isExcelPath(file.name) || isExcelPath((file as FileWithPath).path ?? ''))
    }
    const items = Array.from(data.items)
    if (items.length === 0) {
      return true
    }
    return items.some(item => item.kind === 'file')
  }

  function onDragEnter(event: DragEvent) {
    event.preventDefault()
    applyHover(namesLookLikeExcel(event.dataTransfer))
  }

  function onDragOver(event: DragEvent) {
    event.preventDefault()
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'copy'
    }
    isOver.value = true
  }

  function onDragLeave(event: DragEvent) {
    const current = event.currentTarget
    const related = event.relatedTarget
    if (
      current instanceof Node
      && related instanceof Node
      && current.contains(related)
    ) {
      return
    }
    resetHover()
  }

  function onDrop(event: DragEvent) {
    event.preventDefault()
    resetHover()
    if (isTauri()) {
      return
    }
    const data = event.dataTransfer
    const paths = pathsFromDataTransfer(data)
    if (paths.length > 0) {
      emitPaths(paths)
      return
    }
    const files = Array.from(data?.files ?? [])
    if (files.some(file => isExcelPath(file.name))) {
      options.onInvalid?.(options.missingPathMessage ?? DEFAULT_MISSING_PATH_MESSAGE)
      return
    }
    options.onInvalid?.(INVALID_TYPE_MESSAGE)
  }

  let unlisten: UnlistenFn | undefined
  let cancelled = false

  onMounted(() => {
    if (!isTauri()) {
      return
    }
    void getCurrentWebview()
      .onDragDropEvent((event) => {
        const payload = event.payload
        if (payload.type === 'enter') {
          applyHover(payload.paths.some(isExcelPath))
          return
        }
        if (payload.type === 'over') {
          if (!isOver.value) {
            isValid.value = true
          }
          isOver.value = true
          return
        }
        if (payload.type === 'drop') {
          const paths = payload.paths
          resetHover()
          emitPaths(paths)
          return
        }
        resetHover()
      })
      .then((fn) => {
        if (cancelled) {
          fn()
          return
        }
        unlisten = fn
      })
      .catch(() => {
        // Browser preview without the Tauri webview runtime.
      })
  })

  onUnmounted(() => {
    cancelled = true
    unlisten?.()
  })

  return {
    isOver,
    isValid,
    onDragEnter,
    onDragOver,
    onDragLeave,
    onDrop,
  }
}
