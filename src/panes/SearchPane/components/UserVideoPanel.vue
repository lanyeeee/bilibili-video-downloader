<script setup lang="ts">
import { commands, GetUserVideoInfoParams, UserVideoSearchResult } from '../../../bindings.ts'
import { computed, inject, ref, watch } from 'vue'
import EpisodeCard, { EpisodeInfo } from './EpisodeCard.vue'
import { useEpisodeCard, useEpisodeDropdown, useEpisodeSelection } from '../../../utils.tsx'
import { SelectionArea } from '@viselect/vue'
import { searchPaneRefKey } from '../../../injection_keys.ts'

const userVideoResult = defineModel<UserVideoSearchResult>('userVideoResult', { required: true })

const searchPaneRef = inject(searchPaneRefKey)

const currentPage = ref<number>(1)
const pageCount = computed<number>(() => {
  const count = userVideoResult.value.page.count
  const ps = userVideoResult.value.page.ps
  return Math.ceil(count / ps)
})

const { selectedIds, updateSelectedIds, unselectAll } = useEpisodeSelection()
const selectionAreaRef = ref<InstanceType<typeof SelectionArea>>()
const checkedIds = ref<Set<number>>(new Set())

watch(userVideoResult, () => {
  currentPage.value = userVideoResult.value.page.pn

  selectedIds.value.clear()
  checkedIds.value.clear()
  selectionAreaRef.value?.selection?.clearSelection()
  selectionAreaRef.value?.$el.scrollTo({ top: 0, behavior: 'instant' })
})

const episodeCardRefs = ref<InstanceType<typeof EpisodeCard>[]>([])
const episodeCardRefsMap = computed<Map<number, InstanceType<typeof EpisodeCard>>>(() => {
  const map = new Map<number, InstanceType<typeof EpisodeCard>>()
  episodeCardRefs.value.forEach((card) => map.set(card.episodeInfo.aid, card))
  return map
})

const { dropdownX, dropdownY, dropdownShowing, dropdownOptions, showDropdown } = useEpisodeDropdown(
  () => {
    selectedIds.value.forEach((aid) => checkedIds.value.add(aid))
    dropdownShowing.value = false
  },
  () => {
    selectedIds.value.forEach((aid) => checkedIds.value.delete(aid))
    dropdownShowing.value = false
  },
  () => {
    userVideoResult.value.list.vlist.forEach((ep) => selectedIds.value.add(ep.aid))
    dropdownShowing.value = false
  },
)

const { downloadEpisode, checkboxChecked, handleCheckboxClick, handleContextMenu } = useEpisodeCard(
  async (episodeInfo: EpisodeInfo) => {
    await downloadNormalEpisode(episodeInfo.aid)
  },
  (episodeInfo: EpisodeInfo) => {
    return checkedIds.value.has(episodeInfo.aid)
  },
  (episodeInfo: EpisodeInfo) => {
    const checked = checkedIds.value.has(episodeInfo.aid)
    if (checked) {
      checkedIds.value.delete(episodeInfo.aid)
    } else {
      checkedIds.value.add(episodeInfo.aid)
    }
  },
  (episodeInfo: EpisodeInfo) => {
    if (selectedIds.value.has(episodeInfo.aid)) {
      return
    }
    selectedIds.value.clear()
    selectedIds.value.add(episodeInfo.aid)
    const selection = selectionAreaRef.value?.selection
    if (selection) {
      selection.clearSelection()
      selection.select(`[data-key="${episodeInfo.aid}"]`)
    }
  },
)

async function downloadNormalEpisode(aid: number) {
  // 获取普通视频信息，用于创建下载任务
  const getNormalInfoResult = await commands.getNormalInfo({ Aid: aid })
  if (getNormalInfoResult.status === 'error') {
    console.error(getNormalInfoResult.error)
    return
  }
  // 创建下载任务
  await commands.createDownloadTasks({ Normal: { info: getNormalInfoResult.data, aid_cid_pairs: [[aid, null]] } })
}

async function downloadCheckedEpisodes() {
  for (const aid of checkedIds.value) {
    // 创建下载任务
    await downloadNormalEpisode(aid)
    // 播放下载动画
    const card = episodeCardRefsMap.value.get(aid)
    if (card !== undefined) {
      card.playDownloadAnimation()
    }
    await new Promise((resolve) => setTimeout(resolve, 200))
  }
}

async function getUserVideoInfo(page: number) {
  currentPage.value = page
  const mid = userVideoResult.value.list.vlist[0].mid
  const params: GetUserVideoInfoParams = { mid, pn: page }
  const result = await commands.getUserVideoInfo(params)
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
  userVideoResult.value = result.data
}
</script>

<template>
  <div class="flex flex-col h-full select-none overflow-auto">
    <SelectionArea
      ref="selectionAreaRef"
      class="selection-container flex flex-col flex-1 px-2 overflow-auto"
      :options="{ selectables: '.selectable', features: { deselectOnBlur: true } }"
      @contextmenu="showDropdown"
      @move="updateSelectedIds"
      @start="unselectAll">
      <div class="animate-pulse text-violet">左键拖动进行框选，右键打开菜单</div>
      <div class="flex flex-wrap gap-2">
        <EpisodeCard
          ref="episodeCardRefs"
          v-for="ep in userVideoResult.list.vlist"
          :key="ep.aid"
          :data-key="ep.aid"
          :class="['selectable', selectedIds.has(ep.aid) ? 'selected shadow-md' : 'hover:bg-gray-1']"
          :search-result="userVideoResult"
          :episode="ep"
          :episode-type="'UserVideo'"
          :download-episode="downloadEpisode"
          :checkbox-checked="checkboxChecked"
          :handle-checkbox-click="handleCheckboxClick"
          :handle-context-menu="handleContextMenu"
          :search="searchPaneRef?.search" />
      </div>
    </SelectionArea>

    <div class="flex gap-2 m-2 box-border">
      <n-pagination :page-count="pageCount" :page="currentPage" @update:page="getUserVideoInfo($event)" />
      <n-button class="ml-auto" size="small" type="primary" @click="downloadCheckedEpisodes">下载勾选视频</n-button>
    </div>

    <n-dropdown
      placement="bottom-start"
      trigger="manual"
      :x="dropdownX"
      :y="dropdownY"
      :options="dropdownOptions"
      :show="dropdownShowing"
      :on-clickoutside="() => (dropdownShowing = false)" />
  </div>
</template>

<style scoped>
.selection-container .selected {
  @apply bg-[rgb(204,232,255)];
}
</style>
