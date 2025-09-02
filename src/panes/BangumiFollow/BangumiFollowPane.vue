<script setup lang="ts">
import { useStore } from '../../store.ts'
import { ref, watch } from 'vue'
import { BangumiFollowInfo, commands } from '../../bindings.ts'
import BangumiFollowPanel from './components/BangumiFollowPanel.vue'

const store = useStore()

const bangumiFollowInfo = ref<BangumiFollowInfo>()

watch(
  () => store.userInfo,
  async () => {
    if (store.userInfo === undefined) {
      bangumiFollowInfo.value = undefined
      return
    }

    const result = await commands.getBangumiFollowInfo({
      vmid: store.userInfo.mid,
      pn: 1,
      type: 1,
      follow_status: 0,
    })
    if (result.status === 'error') {
      console.error(result.error)
      return
    }
    bangumiFollowInfo.value = result.data
  },
)
</script>

<template>
  <div v-if="bangumiFollowInfo !== undefined" class="h-full">
    <BangumiFollowPanel v-model:bangumi-follow-info="bangumiFollowInfo" />
  </div>
  <n-empty v-else class="mt-2" description="请先登录" />
</template>
