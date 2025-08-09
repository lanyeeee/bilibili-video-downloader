<script setup lang="tsx">
import { BangumiSearchResult, commands, EpInBangumi } from '../../../bindings.ts'
import { SelectionArea } from '@viselect/vue'
import { ref, nextTick, watch, computed } from 'vue'
import CollectionCard from './CollectionCard.vue'
import { useEpisodeCard, useEpisodeDropdown, useEpisodeSelection } from '../../../utils.tsx'
import EpisodeCard, { EpisodeInfo } from './EpisodeCard.vue'

const props = defineProps<{
  bangumiResult: BangumiSearchResult
}>()

const collectionCardShowing = ref<boolean>(false)

const { selectedIds, updateSelectedIds, unselectAll } = useEpisodeSelection()
const selectionAreaRef = ref<InstanceType<typeof SelectionArea>>()
const checkedIds = ref<Set<number>>(new Set())

const rootDivRef = ref<HTMLDivElement>()
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
    showingEpisodes.value.forEach((ep) => selectedIds.value.add(ep.aid))
    dropdownShowing.value = false
  },
)

const { downloadEpisode, checkboxChecked, handleCheckboxClick, handleContextMenu } = useEpisodeCard(
  async (episodeInfo: EpisodeInfo) => {
    if (episodeInfo.epId !== undefined && episodeInfo.epId !== 0) {
      // 创建番剧下载任务
      await downloadBangumiEpisode(episodeInfo.epId)
      return
    } else {
      await downloadNormalEpisode(episodeInfo.aid)
    }
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

const currentTabIndex = ref<number>(0)

const tabNames = computed<string[]>(() => {
  const sections = props.bangumiResult.info.section
  if (sections === null) {
    return ['正片']
  }
  return ['正片', ...sections.map((section) => section.title)]
})

const showingEpisodes = computed<EpInBangumi[]>(() => {
  if (currentTabIndex.value === 0) {
    return props.bangumiResult.info.episodes
  }
  const sections = props.bangumiResult.info.section
  if (sections === null || currentTabIndex.value - 1 >= sections.length) {
    return []
  }
  return sections[currentTabIndex.value - 1].episodes
})

watch(currentTabIndex, () => {
  selectionAreaRef.value?.$el.scrollTo({ top: 0, behavior: 'instant' })
})

watch(
  () => props.bangumiResult,
  async () => {
    const episode = props.bangumiResult.ep
    if (episode === null) {
      return
    }

    const sections = props.bangumiResult.info.section
    if (sections === null || props.bangumiResult.info.episodes.some((ep) => ep.aid === episode.aid)) {
      currentTabIndex.value = 0
    } else {
      currentTabIndex.value = sections.findIndex((s) => s.episodes.some((ep) => ep.aid === episode.aid)) + 1
    }

    selectedIds.value = new Set([episode.aid])
    checkedIds.value = new Set([episode.aid])
    const selection = selectionAreaRef.value?.selection
    if (selection) {
      selection.clearSelection()
      selection.select(`[data-key="${episode.aid}"]`)
    }

    await nextTick()

    if (rootDivRef.value === undefined) {
      return
    }

    const targetElement = rootDivRef.value.querySelector(`[data-key="${episode.aid}"]`)
    if (targetElement !== null) {
      targetElement.scrollIntoView({ behavior: 'smooth', block: 'center' })
    }
  },
  { immediate: true },
)

async function downloadCheckedEpisodes() {
  for (const aid of checkedIds.value) {
    const ep = showingEpisodes.value.find((ep) => ep.aid === aid)
    if (ep === undefined) {
      continue
    }
    if (ep.link_type === null) {
      await downloadBangumiEpisode(ep.ep_id)
      playCardDownloadAnimation(ep.aid)
    } else {
      await downloadNormalEpisode(ep.aid)
      playCardDownloadAnimation(ep.aid)
      await new Promise((resolve) => setTimeout(resolve, 200))
    }
  }
}

async function downloadBangumiEpisode(epId: number) {
  await commands.createDownloadTasks({ Bangumi: { ep_ids: [epId], info: props.bangumiResult.info } })
}

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

function playCardDownloadAnimation(aid: number) {
  const card = episodeCardRefsMap.value.get(aid)
  if (card !== undefined) {
    card.playDownloadAnimation()
  }
}
</script>

<template>
  <div class="flex flex-col h-full select-none" ref="rootDivRef">
    <SelectionArea
      ref="selectionAreaRef"
      class="selection-container flex flex-col flex-1 px-2 pt-0 overflow-auto"
      :options="{ selectables: '.selectable', features: { deselectOnBlur: true } }"
      @contextmenu="showDropdown"
      @move="updateSelectedIds"
      @start="unselectAll">
      <div class="animate-pulse text-violet">左键拖动进行框选，右键打开菜单，滚轮可以滚动底部的标签</div>
      <div class="flex flex-wrap gap-2">
        <EpisodeCard
          ref="episodeCardRefs"
          v-for="ep in showingEpisodes"
          :key="ep.aid"
          :data-key="ep.aid"
          :class="['selectable', selectedIds.has(ep.aid) ? 'selected shadow-md' : 'hover:bg-gray-1']"
          :search-result="bangumiResult"
          :episode="ep"
          :episode-type="'Bangumi'"
          :download-episode="downloadEpisode"
          :checkbox-checked="checkboxChecked"
          :handle-checkbox-click="handleCheckboxClick"
          :handle-context-menu="handleContextMenu" />
      </div>
    </SelectionArea>

    <n-tabs class="select-none mt-2" v-model:value="currentTabIndex" type="line" size="small" placement="bottom">
      <n-tab v-for="(tabName, index) in tabNames" :key="index" :name="index" :tab="tabName" />

      <template #suffix>
        <n-button class="ml-auto mb-2" size="small" @click="collectionCardShowing = !collectionCardShowing">
          {{ collectionCardShowing ? '隐藏合集' : '显示合集' }}
        </n-button>
        <n-button class="mx-2 mb-2" size="small" type="primary" @click="downloadCheckedEpisodes">下载勾选视频</n-button>
      </template>
    </n-tabs>

    <n-collapse-transition :show="collectionCardShowing">
      <CollectionCard
        class="mt-0"
        :title="bangumiResult.info.title"
        :description="bangumiResult.info.evaluate"
        :cover="bangumiResult.info.cover"
        :up-name="bangumiResult.info.up_info?.uname"
        :up-avatar="bangumiResult.info.up_info?.avatar"
        :up-uid="bangumiResult.info.up_info?.mid" />
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
.selection-container .selected {
  @apply bg-[rgb(204,232,255)];
}

:deep(.n-tabs-nav__suffix) {
  @apply important-border-0;
}
</style>
