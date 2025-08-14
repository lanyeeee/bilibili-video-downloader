<script setup lang="ts">
import { commands, MediaInWatchLater, WatchLaterInfo } from '../../../bindings.ts'
import { computed, inject, ref, watch } from 'vue'
import { useEpisodeDropdown, useEpisodeSelection } from '../../../utils.tsx'
import { SelectionArea } from '@viselect/vue'
import { searchPaneRefKey } from '../../../injection_keys.ts'
import WatchLaterCard from './WatchLaterCard.vue'

const watchLaterInfo = defineModel<WatchLaterInfo>('watchLaterInfo', { required: true })

const searchPaneRef = inject(searchPaneRefKey)

const currentPage = ref<number>(1)

const pageCount = computed<number>(() => {
  if (watchLaterInfo.value === undefined) {
    return 1
  }
  return Math.ceil(watchLaterInfo.value.count / 20)
})

const watchLaterCardRefs = ref<InstanceType<typeof WatchLaterCard>[]>([])
const watchLaterCardRefsMap = computed<Map<number, InstanceType<typeof WatchLaterCard>>>(() => {
  const map = new Map<number, InstanceType<typeof WatchLaterCard>>()
  watchLaterCardRefs.value.forEach((card) => map.set(card.media.aid, card))
  return map
})

const { selectedIds, updateSelectedIds, unselectAll } = useEpisodeSelection()
const selectionAreaRef = ref<InstanceType<typeof SelectionArea>>()
const checkedIds = ref<Set<number>>(new Set())

watch(watchLaterInfo, () => {
  selectedIds.value.clear()
  checkedIds.value.clear()
  selectionAreaRef.value?.selection?.clearSelection()
  selectionAreaRef.value?.$el.scrollTo({ top: 0, behavior: 'instant' })
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
    watchLaterInfo.value.list?.forEach((media) => selectedIds.value.add(media.aid))
    dropdownShowing.value = false
  },
)

const { downloadEpisode, checkboxChecked, handleCheckboxClick, handleContextMenu } = useWatchLaterCard(
  async (media: MediaInWatchLater) => {
    await downloadNormalEpisode(media.aid)
  },
  (media: MediaInWatchLater) => {
    return checkedIds.value.has(media.aid)
  },
  (media: MediaInWatchLater) => {
    const checked = checkedIds.value.has(media.aid)
    if (checked) {
      checkedIds.value.delete(media.aid)
    } else {
      checkedIds.value.add(media.aid)
    }
  },
  (media: MediaInWatchLater) => {
    if (selectedIds.value.has(media.aid)) {
      return
    }

    selectedIds.value.clear()
    selectedIds.value.add(media.aid)
    const selection = selectionAreaRef.value?.selection
    if (selection) {
      selection.clearSelection()
      selection.select(`[data-key="${media.aid}"]`)
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
    const card = watchLaterCardRefsMap.value.get(aid)
    if (card !== undefined) {
      card.playDownloadAnimation()
    }
    await new Promise((resolve) => setTimeout(resolve, 200))
  }
}

async function getWatchLaterInfo(page: number) {
  currentPage.value = page
  const result = await commands.getWatchLaterInfo(page)
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
  watchLaterInfo.value = result.data
}

function useWatchLaterCard(
  downloadEpisode: (media: MediaInWatchLater) => Promise<void>,
  checkboxChecked: (media: MediaInWatchLater) => boolean,
  handleCheckboxClick: (media: MediaInWatchLater) => void,
  handleContextMenu: (media: MediaInWatchLater) => void,
) {
  return {
    downloadEpisode,
    checkboxChecked,
    handleCheckboxClick,
    handleContextMenu,
  }
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
        <template v-for="media in watchLaterInfo.list" :key="media.aid">
          <WatchLaterCard
            v-if="media.redirect_url === null"
            ref="watchLaterCardRefs"
            :data-key="media.aid"
            :class="[
              'selectable border border-solid border-transparent',
              selectedIds.has(media.aid) ? 'selected shadow-md' : 'hover:bg-gray-1',
            ]"
            :media="media"
            :download-episode="downloadEpisode"
            :checkbox-checked="checkboxChecked"
            :handle-checkbox-click="handleCheckboxClick"
            :handle-context-menu="handleContextMenu"
            :search="searchPaneRef?.search" />
          <WatchLaterCard
            v-else
            ref="favCardRefs"
            class="border border-solid border-transparent hover:border-gray-3"
            :media="media"
            :search="searchPaneRef?.search" />
        </template>
      </div>
    </SelectionArea>

    <div class="flex gap-2 m-2 box-border">
      <n-pagination :page-count="pageCount" :page="currentPage" @update:page="getWatchLaterInfo($event)" />
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
