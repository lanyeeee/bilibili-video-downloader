import { defineStore } from 'pinia'
import { ref } from 'vue'
import { Config, UserInfo } from './bindings.ts'

export const useStore = defineStore('store', () => {
  const config = ref<Config>()
  const userInfo = ref<UserInfo>()

  return { config, userInfo }
})
