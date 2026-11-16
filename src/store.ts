import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { Config, UserInfo } from './bindings.ts'
import { CurrentNavName } from './AppContent.vue'
import { createProgresses } from './progresses.ts'

export const useStore = defineStore('store', () => {
  const config = ref<Config>()
  const userInfo = ref<UserInfo>()
  const currentNavName = ref<CurrentNavName>('search')
  const downloadSpeed = ref<string>('')

  const { progresses, completedProgresses, uncompletedProgresses, setProgress, patchProgress, deleteProgress } =
    createProgresses()
  const uncompletedProgressesCount = computed<number>(() => {
    return uncompletedProgresses.value.size
  })

  return {
    config,
    currentNavName,
    userInfo,
    progresses,
    completedProgresses,
    uncompletedProgresses,
    setProgress,
    patchProgress,
    deleteProgress,
    uncompletedProgressesCount,
    downloadSpeed,
  }
})
