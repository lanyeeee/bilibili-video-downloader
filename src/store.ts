import { defineStore } from 'pinia'
import { ref, computed, readonly } from 'vue'
import { Config, UserInfo } from './bindings.ts'
import { CurrentNavName } from './AppContent.vue'
import { ProgressData } from './panes/DownloadPane/DownloadPane.vue'

export const useStore = defineStore('store', () => {
  const config = ref<Config>()
  const userInfo = ref<UserInfo>()
  const currentNavName = ref<CurrentNavName>('search')
  const downloadSpeed = ref<string>('')

  const { progresses, updateProgresses } = useProgresses()
  const uncompletedProgressesCount = computed<number>(() => {
    return Array.from(progresses.value.values()).filter(({ state }) => state !== 'Completed').length
  })

  return {
    config,
    currentNavName,
    userInfo,
    progresses,
    updateProgresses,
    uncompletedProgressesCount,
    downloadSpeed,
  }
})

function useProgresses() {
  // 内部的高频更新状态
  const _progresses = new Map<string, ProgressData>()
  // 对外暴露的响应式状态
  const progresses = ref<Map<string, ProgressData>>(new Map())

  // 用于确保在同一渲染帧内只安排一次UI更新
  let isUpdateScheduled = false

  // 将 `_progresses` 的内容更新到 `progresses` 中，并触发重新渲染
  const updateProgressesOnFrame = () => {
    const newProgressesMap = new Map<string, ProgressData>()
    for (const [key, value] of _progresses.entries()) {
      newProgressesMap.set(key, { ...value })
    }
    progresses.value = newProgressesMap

    isUpdateScheduled = false
  }

  const updateProgresses = (updateFn: (progresses: Map<string, ProgressData>) => void) => {
    // 使用传入的更新函数来修改 `_progresses`
    updateFn(_progresses)

    if (!isUpdateScheduled) {
      // 如果没有安排过UI更新，则安排一次
      isUpdateScheduled = true
      // 使用 `requestAnimationFrame` 调度 UI 更新
      requestAnimationFrame(updateProgressesOnFrame)
    }
  }

  return { progresses: readonly(progresses), updateProgresses }
}
