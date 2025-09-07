<script setup lang="ts">
import { commands, DeviceType, HistoryDetail, HistoryInfo } from '../../../bindings.ts'
import { computed, inject, ref, watch } from 'vue'
import { searchPaneRefKey } from '../../../injection_keys.ts'
import HistoryCard from './HistoryCard.vue'
import { useEpisodeDropdown, useEpisodeSelection } from '../../../utils.tsx'
import { SelectionArea } from '@viselect/vue'
import FloatLabelInput from '../../../components/FloatLabelInput.vue'
import { PhMagnifyingGlass } from '@phosphor-icons/vue'

const historyInfo = defineModel<HistoryInfo>('historyInfo', { required: true })

const searchPaneRef = inject(searchPaneRefKey)

const currentPage = ref<number>(1)
const pageCount = computed<number>(() => Math.ceil(historyInfo.value.page.total / 20))

const searching = ref<boolean>(false)

const searchInput = ref<string>('')

type DurationTabName = 'all' | '<10' | '10-30' | '30-60' | '>60'
const durationTabName = ref<DurationTabName>('all')
let addTimeStart: number = 0
let addTimeEnd: number = 0

const datePickerRange = ref<[number, number]>(getInitRange())
type StartTimeTabName = 'all' | 'today' | 'yesterday' | 'week' | 'date-picker'
const startTimeTabName = ref<StartTimeTabName>('all')
let arcMinDuration: number = 0
let arcMaxDuration: number = 0

const selectedDeviceType = ref<DeviceType>('All')

watch(durationTabName, () => {
  if (durationTabName.value === 'all') {
    arcMinDuration = 0
    arcMaxDuration = 0
  } else if (durationTabName.value === '<10') {
    arcMinDuration = 0
    arcMaxDuration = 10 * 60
  } else if (durationTabName.value === '10-30') {
    arcMinDuration = 10 * 60
    arcMaxDuration = 30 * 60
  } else if (durationTabName.value === '30-60') {
    arcMinDuration = 30 * 60
    arcMaxDuration = 60 * 60
  } else if (durationTabName.value === '>60') {
    arcMinDuration = 60 * 60
    arcMaxDuration = 0
  }

  getHistory(1)
})

watch(startTimeTabName, () => {
  const tabName = startTimeTabName.value
  if (tabName === 'date-picker') {
    return
  }

  if (tabName === 'all') {
    addTimeStart = 0
    addTimeEnd = 0
  } else if (tabName === 'today') {
    const now = new Date()
    addTimeStart = Math.floor(new Date(now.setHours(0, 0, 0, 0)).getTime() / 1000)
    addTimeEnd = 0
  } else if (tabName === 'yesterday') {
    const now = new Date()
    const yesterday = new Date(now.setDate(now.getDate() - 1))
    addTimeStart = Math.floor(new Date(yesterday.setHours(0, 0, 0, 0)).getTime() / 1000)
    addTimeEnd = Math.floor(new Date(yesterday.setHours(23, 59, 59, 0)).getTime() / 1000)
  } else if (tabName === 'week') {
    const now = new Date()
    const weekAgo = new Date(now.setDate(now.getDate() - 7))
    addTimeStart = Math.floor(new Date(weekAgo.setHours(0, 0, 0, 0)).getTime() / 1000)
    addTimeEnd = 0
  }

  getHistory(1)
})

watch(selectedDeviceType, () => getHistory(1))

async function getHistory(page: number) {
  currentPage.value = page
  searching.value = true

  const result = await commands.getHistoryInfo({
    pn: page,
    keyword: searchInput.value,
    add_time_start: addTimeStart,
    add_time_end: addTimeEnd,
    arc_min_duration: arcMinDuration,
    arc_max_duration: arcMaxDuration,
    device_type: selectedDeviceType.value,
  })
  if (result.status === 'error') {
    console.error(result.error)
    searching.value = false
    return
  }
  historyInfo.value = result.data
  searching.value = false
}

const historyCardRefs = ref<InstanceType<typeof HistoryCard>[]>([])
const historyCardRefsMap = computed<Map<number, InstanceType<typeof HistoryCard>>>(() => {
  const map = new Map<number, InstanceType<typeof HistoryCard>>()
  historyCardRefs.value.forEach((card) => map.set(card.historyDetail.kid, card))
  return map
})

const { selectedIds, updateSelectedIds, unselectAll } = useEpisodeSelection()
const selectionAreaRef = ref<InstanceType<typeof SelectionArea>>()
const checkedIds = ref<Set<number>>(new Set())

watch(historyInfo, () => {
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
    historyInfo.value.list?.forEach((detail) => selectedIds.value.add(detail.kid))
    dropdownShowing.value = false
  },
)

const { downloadEpisode, checkboxChecked, handleCheckboxClick, handleContextMenu } = useFavCard(
  async (historyDetail: HistoryDetail) => {
    await downloadNormalEpisode(historyDetail.kid)
  },
  (historyDetail: HistoryDetail) => {
    return checkedIds.value.has(historyDetail.kid)
  },
  (historyDetail: HistoryDetail) => {
    const checked = checkedIds.value.has(historyDetail.kid)
    if (checked) {
      checkedIds.value.delete(historyDetail.kid)
    } else {
      checkedIds.value.add(historyDetail.kid)
    }
  },
  (historyDetail: HistoryDetail) => {
    if (selectedIds.value.has(historyDetail.kid)) {
      return
    }

    selectedIds.value.clear()
    selectedIds.value.add(historyDetail.kid)
    const selection = selectionAreaRef.value?.selection
    if (selection) {
      selection.clearSelection()
      selection.select(`[data-key="${historyDetail.kid}"]`)
    }
  },
)

async function downloadNormalEpisode(aid: number) {
  // 获取普通视频信息，用于创建下载任务
  const result = await commands.getNormalInfo({ Aid: aid })
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
  // 创建下载任务
  await commands.createDownloadTasks({ Normal: { info: result.data, aid_cid_pairs: [[aid, null]] } })
}

async function downloadCheckedEpisodes() {
  for (const aid of checkedIds.value) {
    // 创建下载任务
    await downloadNormalEpisode(aid)
    // 播放下载动画
    const card = historyCardRefsMap.value.get(aid)
    if (card !== undefined) {
      card.playDownloadAnimation()
    }
    await new Promise((resolve) => setTimeout(resolve, 200))
  }
}

function handleTimePickerConfirm(range: [number, number] | null) {
  if (range === null) {
    return
  }

  addTimeStart = Math.floor(new Date(range[0]).setHours(0, 0, 0, 0) / 1000)
  addTimeEnd = Math.floor(new Date(range[1]).setHours(23, 59, 59, 0) / 1000)

  startTimeTabName.value = 'date-picker'
  getHistory(1)
}

function getInitRange(): [number, number] {
  const now = new Date()

  const lastMonthStart = new Date(now.getFullYear(), now.getMonth() - 1, 1)
  lastMonthStart.setHours(0, 0, 0, 0)

  const todayEnd = new Date(now)
  todayEnd.setHours(23, 59, 59, 999)

  return [lastMonthStart.getTime(), todayEnd.getTime()]
}

function useFavCard(
  downloadEpisode: (historyDetail: HistoryDetail) => Promise<void>,
  checkboxChecked: (historyDetail: HistoryDetail) => boolean,
  handleCheckboxClick: (historyDetail: HistoryDetail) => void,
  handleContextMenu: (historyDetail: HistoryDetail) => void,
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
    <n-input-group class="box-border px-2 pt-2">
      <FloatLabelInput
        label="搜索标题/up主昵称"
        size="small"
        v-model:value="searchInput"
        clearable
        @keydown.enter="getHistory(1)" />
      <n-popover trigger="click" :show-arrow="false">
        <template #trigger>
          <n-select class="w-20%" :show="false" default-value="更多筛选" size="small" />
        </template>
        <div class="w-155">
          <n-tabs class="w-111.5" type="segment" size="small" v-model:value="durationTabName">
            <n-tab name="all">全部时长</n-tab>
            <n-tab name="<10">10分钟以下</n-tab>
            <n-tab name="10-30">10-30分钟</n-tab>
            <n-tab name="30-60">30-60分钟</n-tab>
            <n-tab name=">60">60分钟以上</n-tab>
          </n-tabs>
          <div class="flex items-center">
            <n-tabs
              class="justify-between"
              type="segment"
              size="small"
              v-model:value="startTimeTabName"
              @before-leave="(tabName: StartTimeTabName) => tabName !== 'date-picker'">
              <n-tab name="all">全部时间</n-tab>
              <n-tab name="today">今天</n-tab>
              <n-tab name="yesterday">昨天</n-tab>
              <n-tab name="week">近一周</n-tab>
              <n-tab class="cursor-default!" name="date-picker">
                <n-date-picker
                  size="small"
                  class="ml-auto w-63.5 px-1"
                  v-model:value="datePickerRange"
                  type="daterange"
                  @confirm="handleTimePickerConfirm" />
              </n-tab>
            </n-tabs>
          </div>
          <n-tabs class="w-111.5" type="segment" size="small" v-model:value="selectedDeviceType">
            <n-tab name="All">全部设备</n-tab>
            <n-tab name="PC">PC</n-tab>
            <n-tab name="Mobile">手机</n-tab>
            <n-tab name="TV">平板</n-tab>
            <n-tab name="Pad">TV</n-tab>
          </n-tabs>
        </div>
      </n-popover>
      <n-button :loading="searching" type="primary" size="small" class="w-10%" @click="getHistory(1)">
        <template #icon>
          <n-icon size="22">
            <PhMagnifyingGlass weight="bold" />
          </n-icon>
        </template>
      </n-button>
    </n-input-group>

    <SelectionArea
      ref="selectionAreaRef"
      class="selection-container flex flex-col flex-1 px-2 overflow-auto"
      :options="{ selectables: '.selectable', features: { deselectOnBlur: true } }"
      @contextmenu="showDropdown"
      @move="updateSelectedIds"
      @start="unselectAll">
      <div class="animate-pulse text-violet">左键拖动进行框选，右键打开菜单</div>
      <div class="flex flex-wrap gap-2">
        <template v-for="historyDetail in historyInfo.list" :key="historyDetail.kid">
          <HistoryCard
            v-if="historyDetail.badge === ''"
            ref="historyCardRefs"
            :data-key="historyDetail.kid"
            :class="[
              'selectable border border-solid border-transparent',
              selectedIds.has(historyDetail.kid) ? 'selected shadow-md' : 'hover:bg-gray-1',
            ]"
            episode-type="Normal"
            :history-detail="historyDetail"
            :download-episode="downloadEpisode"
            :checkbox-checked="checkboxChecked"
            :handle-checkbox-click="handleCheckboxClick"
            :handle-context-menu="handleContextMenu"
            :search="searchPaneRef?.search" />

          <HistoryCard
            v-else-if="historyDetail.badge === '课堂'"
            ref="historyCardRefs"
            class="border border-solid border-transparent hover:border-gray-3"
            episode-type="Cheese"
            :history-detail="historyDetail"
            :search="searchPaneRef?.search" />

          <HistoryCard
            v-else
            ref="historyCardRefs"
            class="border border-solid border-transparent hover:border-gray-3"
            episode-type="Bangumi"
            :history-detail="historyDetail"
            :search="searchPaneRef?.search" />
        </template>
      </div>
    </SelectionArea>

    <div class="flex gap-2 m-2 box-border">
      <n-pagination :page-count="pageCount" :page="currentPage" @update:page="getHistory($event)" />
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
