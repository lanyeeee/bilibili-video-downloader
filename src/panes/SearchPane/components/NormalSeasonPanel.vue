<script setup lang="tsx">
import { commands, NormalSearchResult, UgcSeason, SectionInNormal } from '../../../bindings.ts'
import { SelectionArea } from '@viselect/vue'
import { ref, nextTick, computed, watch } from 'vue'
import CollectionCard from './CollectionCard.vue'
import { useEpisodeCard, useEpisodeDropdown, useEpisodeSelection } from '../../../utils.tsx'
import { TabsInst } from 'naive-ui'
import EpisodeCard, { EpisodeInfo } from './EpisodeCard.vue'

const props = defineProps<{
  ugcSeason: UgcSeason
  normalResult: NormalSearchResult
}>()

const collectionCardShowing = ref<boolean>(false)

const tabsInstRef = ref<TabsInst>()

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
    currentSection.value.episodes.forEach((ep) => selectedIds.value.add(ep.aid))
    dropdownShowing.value = false
  },
)

const { downloadEpisode, checkboxChecked, handleCheckboxClick, handleContextMenu } = useEpisodeCard(
  async (episodeInfo: EpisodeInfo) => {
    await commands.createDownloadTasks({
      Normal: {
        info: props.normalResult,
        aid_cid_pairs: [[episodeInfo.aid, null]],
      },
    })
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

const currentSectionIndex = ref<number>(0)
const currentSection = computed<SectionInNormal>(() => {
  return props.ugcSeason.sections[currentSectionIndex.value]
})

watch(currentSectionIndex, () => {
  selectionAreaRef.value?.$el.scrollTo({ top: 0, behavior: 'instant' })
})

watch(
  () => props.normalResult,
  async () => {
    currentSectionIndex.value = props.ugcSeason.sections.findIndex((s) =>
      s.episodes.some((ep) => ep.aid === props.normalResult.aid),
    )

    selectedIds.value = new Set([props.normalResult.aid])
    checkedIds.value = new Set([props.normalResult.aid])
    const selection = selectionAreaRef.value?.selection
    if (selection) {
      selection.clearSelection()
      selection.select(`[data-key="${props.normalResult.aid}"]`)
    }

    await nextTick()
    tabsInstRef.value?.syncBarPosition()

    if (rootDivRef.value === undefined) {
      return
    }

    const targetElement = rootDivRef.value.querySelector(`[data-key="${props.normalResult.aid}"]`)
    if (targetElement) {
      targetElement.scrollIntoView({ behavior: 'smooth', block: 'center' })
    }
  },
  { immediate: true },
)

async function downloadCheckedEpisodes() {
  const currentSectionAids = new Set(currentSection.value.episodes.map((ep) => ep.aid))
  const aidsToDownload = Array.from(checkedIds.value).filter((aid) => currentSectionAids.has(aid))
  // 创建下载任务
  await commands.createDownloadTasks({
    Normal: {
      info: props.normalResult,
      aid_cid_pairs: aidsToDownload.map((aid) => [aid, null]),
    },
  })
  // 播放下载动画
  for (const aid of aidsToDownload) {
    const card = episodeCardRefsMap.value.get(aid)
    if (card !== undefined) {
      card.playDownloadAnimation()
    }
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
          v-for="ep in currentSection.episodes"
          :key="ep.aid"
          :data-key="ep.aid"
          :class="['selectable', selectedIds.has(ep.aid) ? 'selected shadow-md' : 'hover:bg-gray-1']"
          :search-result="normalResult"
          :episode="ep"
          :episode-type="'NormalSeason'"
          :download-episode="downloadEpisode"
          :checkbox-checked="checkboxChecked"
          :handle-checkbox-click="handleCheckboxClick"
          :handle-context-menu="handleContextMenu" />
      </div>
    </SelectionArea>

    <n-tabs
      ref="tabsInstRef"
      type="line"
      class="select-none mt-2"
      v-model:value="currentSectionIndex"
      size="small"
      placement="bottom">
      <n-tab v-for="(section, index) in ugcSeason.sections" :key="index" :name="index" :tab="section.title" />

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
        :title="ugcSeason.title"
        :description="ugcSeason.intro"
        :cover="ugcSeason.cover"
        :up-name="normalResult.owner.name"
        :up-avatar="normalResult.owner.face"
        :up-uid="normalResult.owner.mid" />
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
