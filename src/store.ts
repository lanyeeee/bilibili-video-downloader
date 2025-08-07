import { defineStore } from 'pinia'
import { ref } from 'vue'
import { Config, UserInfo } from './bindings.ts'
import { CurrentNavName } from './AppContent.vue'

export const useStore = defineStore('store', () => {
  const config = ref<Config>()
  const userInfo = ref<UserInfo>()
  const currentNavName = ref<CurrentNavName>('search')

  return { config, userInfo, currentNavName }
})
