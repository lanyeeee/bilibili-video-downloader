<script setup lang="ts">
import { ref, watch } from 'vue'
import { commands, FavInfo } from '../../bindings.ts'
import { useStore } from '../../store.ts'
import FavPanel from './components/FavPanel.vue'

const store = useStore()

const favInfo = ref<FavInfo>()

watch(
  () => store.userInfo,
  async () => {
    if (store.userInfo === undefined) {
      favInfo.value = undefined
      return
    }

    const getFavFoldersResult = await commands.getFavFolders(store.userInfo.mid)
    if (getFavFoldersResult.status === 'error') {
      console.error(getFavFoldersResult.error)
      return
    }

    const favFolders = getFavFoldersResult.data
    const getFavInfoResult = await commands.getFavInfo({ media_list_id: favFolders.list[0].id, pn: 1 })
    if (getFavInfoResult.status === 'error') {
      console.error(getFavInfoResult.error)
      return
    }
    favInfo.value = getFavInfoResult.data
  },
)
</script>

<template>
  <div v-if="favInfo !== undefined" class="h-full">
    <FavPanel v-model:fav-info="favInfo" />
  </div>
  <n-empty v-else class="mt-2" description="请先登录" />
</template>
