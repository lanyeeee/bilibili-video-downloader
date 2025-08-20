<script setup lang="ts">
import { BangumiFollowInfo, commands, EpInBangumiFollow } from '../../../bindings.ts'
import { computed, ref, watch } from 'vue'
import { useStore } from '../../../store.ts'
import BangumiFollowCard from './BangumiFollowCard.vue'
import { useEpisodeDropdown, useEpisodeSelection } from '../../../utils.tsx'
import { SelectionArea } from '@viselect/vue'
import { SelectOption } from 'naive-ui'

const store = useStore()

const bangumiFollowInfo = defineModel<BangumiFollowInfo>('bangumiFollowInfo', { required: true })

const selectedType = ref<number>(1)
const selectTypeOptions: SelectOption[] = [
  { label: '追番', value: 1 },
  { label: '追剧', value: 2 },
]

const selectedFollowStatus = ref<number>(0)
const selectFollowStatusOptions: SelectOption[] = [
  { label: '全部', value: 0 },
  { label: '想看', value: 1 },
  { label: '在看', value: 2 },
  { label: '看过', value: 3 },
]

const currentPage = ref<number>(1)
const pageCount = computed<number>(() => {
  if (bangumiFollowInfo.value === undefined) {
    return 1
  }
  return Math.ceil(bangumiFollowInfo.value.total / 24)
})

const bangumiFollowCardRefs = ref<InstanceType<typeof BangumiFollowCard>[]>([])
const bangumiFollowCardRefsMap = computed<Map<number, InstanceType<typeof BangumiFollowCard>>>(() => {
  const map = new Map<number, InstanceType<typeof BangumiFollowCard>>()
  bangumiFollowCardRefs.value.forEach((card) => map.set(card.ep.season_id, card))
  return map
})

const { selectedIds, updateSelectedIds, unselectAll } = useEpisodeSelection()
const selectionAreaRef = ref<InstanceType<typeof SelectionArea>>()
const checkedIds = ref<Set<number>>(new Set())

watch(bangumiFollowInfo, () => {
  selectedIds.value.clear()
  checkedIds.value.clear()
  selectionAreaRef.value?.selection?.clearSelection()
  selectionAreaRef.value?.$el.scrollTo({ top: 0, behavior: 'instant' })
})

const { dropdownX, dropdownY, dropdownShowing, dropdownOptions, showDropdown } = useEpisodeDropdown(
  () => {
    selectedIds.value.forEach((seasonId) => checkedIds.value.add(seasonId))
    dropdownShowing.value = false
  },
  () => {
    selectedIds.value.forEach((seasonId) => checkedIds.value.delete(seasonId))
    dropdownShowing.value = false
  },
  () => {
    bangumiFollowInfo.value.list?.forEach((ep) => selectedIds.value.add(ep.season_id))
    dropdownShowing.value = false
  },
)

const { downloadEpisode, checkboxChecked, handleCheckboxClick, handleContextMenu } = useBangumiFollowCard(
  async (ep: EpInBangumiFollow) => {
    await downloadSeason(ep.season_id)
  },
  (ep: EpInBangumiFollow) => {
    return checkedIds.value.has(ep.season_id)
  },
  (ep: EpInBangumiFollow) => {
    const checked = checkedIds.value.has(ep.season_id)
    if (checked) {
      checkedIds.value.delete(ep.season_id)
    } else {
      checkedIds.value.add(ep.season_id)
    }
  },
  (ep: EpInBangumiFollow) => {
    if (selectedIds.value.has(ep.season_id)) {
      return
    }

    selectedIds.value.clear()
    selectedIds.value.add(ep.season_id)
    const selection = selectionAreaRef.value?.selection
    if (selection) {
      selection.clearSelection()
      selection.select(`[data-key="${ep.season_id}"]`)
    }
  },
)

async function getBangumiFollowInfo(page: number) {
  if (store.userInfo === undefined) {
    return
  }

  currentPage.value = page

  const result = await commands.getBangumiFollowInfo({
    vmid: store.userInfo.mid,
    pn: page,
    type: selectedType.value,
    follow_status: selectedFollowStatus.value,
  })
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
  bangumiFollowInfo.value = result.data
}

async function downloadSeason(seasonId: number) {
  // 获取番剧信息，用于创建下载任务
  const result = await commands.getBangumiInfo({ SeasonId: seasonId })
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
  const info = result.data
  // 创建下载任务
  await commands.createDownloadTasks({
    Bangumi: { info, ep_ids: info.episodes.map((ep) => ep.ep_id) },
  })
}

async function downloadCheckedEpisodes() {
  for (const seasonId of checkedIds.value) {
    // 创建下载任务
    await downloadSeason(seasonId)
    // 播放下载动画
    const card = bangumiFollowCardRefsMap.value.get(seasonId)
    if (card !== undefined) {
      card.playDownloadAnimation()
    }
    await new Promise((resolve) => setTimeout(resolve, 200))
  }
}

function useBangumiFollowCard(
  downloadEpisode: (ep: EpInBangumiFollow) => Promise<void>,
  checkboxChecked: (ep: EpInBangumiFollow) => boolean,
  handleCheckboxClick: (ep: EpInBangumiFollow) => void,
  handleContextMenu: (ep: EpInBangumiFollow) => void,
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
        <BangumiFollowCard
          v-for="ep in bangumiFollowInfo.list"
          :key="ep.season_id"
          ref="bangumiFollowCardRefs"
          :data-key="ep.season_id"
          :class="[
            'selectable border border-solid border-transparent',
            selectedIds.has(ep.season_id) ? 'selected shadow-md' : 'hover:bg-gray-1',
          ]"
          :ep="ep"
          :download-episode="downloadEpisode"
          :checkbox-checked="checkboxChecked"
          :handle-checkbox-click="handleCheckboxClick"
          :handle-context-menu="handleContextMenu" />
      </div>
    </SelectionArea>
    <div class="flex gap-2 m-2 box-border">
      <n-pagination :page-count="pageCount" :page="currentPage" @update:page="getBangumiFollowInfo($event)" />
      <n-select
        class="w-20"
        v-model:value="selectedType"
        :options="selectTypeOptions"
        size="small"
        @update:value="getBangumiFollowInfo(1)" />
      <n-select
        class="w-20"
        v-model:value="selectedFollowStatus"
        :options="selectFollowStatusOptions"
        size="small"
        @update:value="getBangumiFollowInfo(1)" />
      <n-button class="ml-auto" size="small" type="primary" @click="downloadCheckedEpisodes">下载勾选剧集</n-button>
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
