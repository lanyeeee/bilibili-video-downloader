import { defineStore } from 'pinia'
import { ref } from 'vue'
import { Config } from './bindings.ts'

export const useStore = defineStore('store', () => {
  const config = ref<Config>()

  return { config }
})
