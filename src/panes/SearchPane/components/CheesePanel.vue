<script setup lang="tsx">
import { CheeseSearchResult, commands } from '../../../bindings.ts'
import { PartialSelectionOptions, SelectionArea } from '@viselect/vue'
import { ref, nextTick, watch, computed, useTemplateRef } from 'vue'
import CollectionCard from './CollectionCard.vue'
import { useEpisodeCard, useEpisodeDropdown, useEpisodeSelection } from '../../../utils.tsx'
import EpisodeCard, { EpisodeInfo } from './EpisodeCard.vue'
import { NButton, NCollapseTransition, NDropdown } from 'naive-ui'
import RefreshButton from '../../../components/RefreshButton.vue'

const props = defineProps<{
  cheeseResult: CheeseSearchResult
  refresh: () => Promise<void>
}>()

const collectionCardShowing = ref<boolean>(false)

const selectionOptions: PartialSelectionOptions = {
  selectables: '.selectable',
  features: { deselectOnBlur: true },
  boundaries: '.cheese-panel-selection-container',
}
const selectionAreaRef = useTemplateRef('selectionAreaRef')
const { selectedIds, updateSelectedIds, unselectAll } = useEpisodeSelection()
const checkedIds = ref<Set<number>>(new Set())

const rootDivRef = useTemplateRef('rootDivRef')
const episodeCardRefs = useTemplateRef('episodeCardRefs')
const episodeCardRefsMap = computed<Map<number, InstanceType<typeof EpisodeCard>>>(() => {
  const map = new Map<number, InstanceType<typeof EpisodeCard>>()
  episodeCardRefs.value?.forEach((card) => {
    if (card !== null && card.episodeInfo.epId !== undefined) {
      map.set(card.episodeInfo.epId, card)
    }
  })
  return map
})

const { dropdownX, dropdownY, dropdownShowing, dropdownOptions, showDropdown } = useEpisodeDropdown(
  () => {
    selectedIds.value.forEach((epId) => checkedIds.value.add(epId))
    dropdownShowing.value = false
  },
  () => {
    selectedIds.value.forEach((epId) => checkedIds.value.delete(epId))
    dropdownShowing.value = false
  },
  () => {
    props.cheeseResult.info.episodes.forEach((ep) => selectedIds.value.add(ep.id))
    dropdownShowing.value = false
  },
)

const { downloadEpisode, checkboxChecked, handleCheckboxClick, handleContextMenu } = useEpisodeCard(
  async (episodeInfo: EpisodeInfo) => {
    if (episodeInfo.epId === undefined) {
      return
    }

    await commands.createDownloadTasks({ Cheese: { ep_ids: [episodeInfo.epId], info: props.cheeseResult.info } })
  },
  (episodeInfo: EpisodeInfo) => {
    if (episodeInfo.epId === undefined) {
      return false
    }

    return checkedIds.value.has(episodeInfo.epId)
  },
  (episodeInfo: EpisodeInfo) => {
    if (episodeInfo.epId === undefined) {
      return
    }

    const checked = checkedIds.value.has(episodeInfo.epId)
    if (checked) {
      checkedIds.value.delete(episodeInfo.epId)
    } else {
      checkedIds.value.add(episodeInfo.epId)
    }
  },
  (episodeInfo: EpisodeInfo) => {
    if (episodeInfo.epId === undefined || selectedIds.value.has(episodeInfo.epId)) {
      return
    }

    selectedIds.value.clear()
    selectedIds.value.add(episodeInfo.epId)
    const selection = selectionAreaRef.value?.selection
    if (selection) {
      selection.clearSelection()
      selection.select(`[data-key="${episodeInfo.epId}"]`)
    }
  },
)

watch(
  () => props.cheeseResult,
  async () => {
    const ep = props.cheeseResult.ep
    if (ep === null) {
      selectedIds.value.clear()
      checkedIds.value.clear()
      selectionAreaRef.value?.selection?.clearSelection()
      selectionAreaRef.value?.$el.scrollTo({ top: 0, behavior: 'instant' })
      return
    }
    selectedIds.value = new Set([ep.id])
    checkedIds.value = new Set([ep.id])
    await nextTick()

    if (rootDivRef.value === null) {
      return
    }

    const targetElement = rootDivRef.value.querySelector(`[data-key="${ep.id}"]`)
    if (targetElement) {
      targetElement.scrollIntoView({ behavior: 'smooth', block: 'center' })
    }
  },
  { immediate: true },
)

async function downloadCheckedEpisodes() {
  // 创建下载任务
  const epIdsToDownload = Array.from(checkedIds.value)
  await commands.createDownloadTasks({
    Cheese: {
      ep_ids: epIdsToDownload,
      info: props.cheeseResult.info,
    },
  })
  // 播放下载动画
  for (const epId of epIdsToDownload) {
    const card = episodeCardRefsMap.value.get(epId)
    console.log(card)
    if (card !== undefined) {
      card.playDownloadAnimation()
    }
  }
}
</script>

<template>
  <div class="flex flex-col h-full select-none" ref="rootDivRef">
    <SelectionArea ref="selectionAreaRef" :options="selectionOptions" @move="updateSelectedIds" @start="unselectAll" />
    <div
      class="cheese-panel-selection-container flex flex-col flex-1 px-2 pt-0 overflow-auto"
      @contextmenu="showDropdown">
      <div class="animate-pulse text-violet">左键拖动进行框选，右键打开菜单</div>
      <div class="flex flex-wrap gap-2">
        <EpisodeCard
          ref="episodeCardRefs"
          v-for="ep in cheeseResult.info.episodes"
          :key="ep.id"
          :data-key="ep.id"
          :class="['selectable', selectedIds.has(ep.id) ? 'selected shadow-md' : 'hover:bg-gray-1']"
          :search-result="cheeseResult"
          :episode="ep"
          :episode-type="'Cheese'"
          :download-episode="downloadEpisode"
          :checkbox-checked="checkboxChecked"
          :handle-checkbox-click="handleCheckboxClick"
          :handle-context-menu="handleContextMenu" />
      </div>
    </div>

    <div class="p-2 ml-auto flex items-center gap-2">
      <n-button size="small" @click="collectionCardShowing = !collectionCardShowing">
        {{ collectionCardShowing ? '隐藏合集' : '显示合集' }}
      </n-button>
      <RefreshButton :refresh="refresh" />
      <n-button size="small" type="primary" @click="downloadCheckedEpisodes">下载勾选视频</n-button>
    </div>
    <n-collapse-transition :show="collectionCardShowing">
      <CollectionCard
        class="mt-0"
        :title="cheeseResult.info.title"
        :description="cheeseResult.info.subtitle"
        :cover="cheeseResult.info.cover"
        :up-name="cheeseResult.info.up_info.uname"
        :up-avatar="cheeseResult.info.up_info.avatar"
        :up-uid="cheeseResult.info.up_info.mid" />
    </n-collapse-transition>

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
.cheese-panel-selection-container .selected {
  @apply bg-[rgb(204,232,255)];
}
</style>
