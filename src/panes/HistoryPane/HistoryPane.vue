<script setup lang="ts">
import { ref, watch } from 'vue'
import { commands, HistoryInfo } from '../../bindings.ts'
import { useStore } from '../../store.ts'
import HistoryPanel from './components/HistoryPanel.vue'

const store = useStore()

const historyInfo = ref<HistoryInfo>()

watch(
  () => store.userInfo,
  async () => {
    if (store.userInfo === undefined) {
      historyInfo.value = undefined
      return
    }

    const result = await commands.getHistoryInfo({
      pn: 1,
      keyword: '',
      add_time_start: 0,
      add_time_end: 0,
      arc_max_duration: 0,
      arc_min_duration: 0,
      device_type: 'All',
    })
    if (result.status === 'error') {
      console.error(result.error)
      return
    }

    historyInfo.value = result.data
  },
)
</script>

<template>
  <div v-if="historyInfo !== undefined" class="h-full">
    <HistoryPanel v-model:history-info="historyInfo" />
  </div>
  <n-empty v-else class="mt-2" description="请先登录" />
</template>
