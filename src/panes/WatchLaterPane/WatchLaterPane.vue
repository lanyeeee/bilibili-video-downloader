<script setup lang="ts">
import { ref, watch } from 'vue'
import { useStore } from '../../store.ts'
import { commands, WatchLaterInfo } from '../../bindings.ts'
import WatchLaterPanel from './components/WatchLaterPanel.vue'

const store = useStore()

const watchLaterInfo = ref<WatchLaterInfo>()

watch(
  () => store.userInfo,
  async () => {
    if (store.userInfo === undefined) {
      watchLaterInfo.value = undefined
      return
    }
    const result = await commands.getWatchLaterInfo(1)
    if (result.status === 'error') {
      console.error(result.error)
      return
    }
    watchLaterInfo.value = result.data
  },
)
</script>

<template>
  <div v-if="watchLaterInfo !== undefined" class="h-full">
    <WatchLaterPanel v-model:watch-later-info="watchLaterInfo" />
  </div>
  <n-empty v-else class="mt-2" description="请先登录" />
</template>
