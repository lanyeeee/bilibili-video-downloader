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
  // 对外暴露的响应式状态
  const progresses = ref<Map<string, ProgressData>>(new Map())

  // 等待在同一渲染帧内执行的更新函数
  const pendingUpdateFns: Array<(progresses: Map<string, ProgressData>) => void> = []

  // 用于确保在同一渲染帧内只安排一次UI更新
  let isUpdateScheduled = false

  // 在同一渲染帧内集中执行等待中的更新函数
  const updateProgressesOnFrame = () => {
    isUpdateScheduled = false

    const updateFns = pendingUpdateFns.splice(0)
    for (const updateFn of updateFns) {
      updateFn(progresses.value)
    }
  }

  const updateProgresses = (updateFn: (progresses: Map<string, ProgressData>) => void) => {
    // 将传入的更新函数添加到等待队列
    pendingUpdateFns.push(updateFn)

    if (!isUpdateScheduled) {
      // 如果没有安排过UI更新，则安排一次
      isUpdateScheduled = true
      // 使用 `requestAnimationFrame` 调度 UI 更新
      requestAnimationFrame(updateProgressesOnFrame)
    }
  }

  return { progresses: readonly(progresses), updateProgresses }
}
