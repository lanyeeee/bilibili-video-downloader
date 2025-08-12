<script setup lang="ts">
import { computed, inject, onMounted, ref, watch } from 'vue'
import { commands, FavFolders, Folder, FavInfo, MediaInFav } from '../../../bindings.ts'
import { SelectionArea } from '@viselect/vue'
import { useEpisodeDropdown, useEpisodeSelection } from '../../../utils.tsx'
import { SelectOption } from 'naive-ui'
import { searchPaneRefKey } from '../../../injection_keys.ts'
import FavCard from './FavCard.vue'

const favInfo = defineModel<FavInfo>('favInfo', { required: true })

const searchPaneRef = inject(searchPaneRefKey)

const favFolders = ref<FavFolders>()
const selectedMediaListId = ref<number>(favInfo.value.info.id)
const currentPage = ref<number>(1)

const selectedFolder = computed<Folder | undefined>(() => {
  return favFolders.value?.list.find((folder) => folder.id === selectedMediaListId.value)
})

const selectOptions = computed<SelectOption[]>(() => {
  if (favFolders.value === undefined) {
    return []
  }
  return favFolders.value.list.map((folder) => ({
    label: folder.title,
    value: folder.id,
  }))
})

const pageCount = computed<number>(() => {
  if (selectedFolder.value === undefined) {
    return 1
  }
  return Math.ceil(selectedFolder.value.media_count / 36)
})

onMounted(async () => {
  const result = await commands.getFavFolders(favInfo.value.info.mid)
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
  favFolders.value = result.data
})

watch(
  () => selectedMediaListId.value,
  () => {
    if (favFolders.value === undefined) {
      return
    }
    getFav(1)
  },
)

async function getFav(page: number) {
  currentPage.value = page
  const result = await commands.getFavInfo({ media_list_id: selectedMediaListId.value, pn: page })
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
  favInfo.value = result.data
}

const favCardRefs = ref<InstanceType<typeof FavCard>[]>([])
const favCardRefsMap = computed<Map<number, InstanceType<typeof FavCard>>>(() => {
  const map = new Map<number, InstanceType<typeof FavCard>>()
  favCardRefs.value.forEach((card) => map.set(card.media.id, card))
  return map
})

const { selectedIds, updateSelectedIds, unselectAll } = useEpisodeSelection()
const selectionAreaRef = ref<InstanceType<typeof SelectionArea>>()
const checkedIds = ref<Set<number>>(new Set())

watch(favInfo, async () => {
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
    favInfo.value.medias
      ?.filter((media) => media.attr === 0 && media.type === 2)
      .forEach((media) => selectedIds.value.add(media.id))
    dropdownShowing.value = false
  },
)

const { downloadEpisode, checkboxChecked, handleCheckboxClick, handleContextMenu } = useFavCard(
  async (media: MediaInFav) => {
    await downloadNormalEpisode(media.id)
  },
  (media: MediaInFav) => {
    return checkedIds.value.has(media.id)
  },
  (media: MediaInFav) => {
    const checked = checkedIds.value.has(media.id)
    if (checked) {
      checkedIds.value.delete(media.id)
    } else {
      checkedIds.value.add(media.id)
    }
  },
  (media: MediaInFav) => {
    if (selectedIds.value.has(media.id)) {
      return
    }

    selectedIds.value.clear()
    selectedIds.value.add(media.id)
    const selection = selectionAreaRef.value?.selection
    if (selection) {
      selection.clearSelection()
      selection.select(`[data-key="${media.id}"]`)
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
    const card = favCardRefsMap.value.get(aid)
    if (card !== undefined) {
      card.playDownloadAnimation()
    }
    await new Promise((resolve) => setTimeout(resolve, 200))
  }
}

function useFavCard(
  downloadEpisode: (media: MediaInFav) => Promise<void>,
  checkboxChecked: (media: MediaInFav) => boolean,
  handleCheckboxClick: (media: MediaInFav) => void,
  handleContextMenu: (media: MediaInFav) => void,
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
        <template v-for="media in favInfo.medias" :key="media.id">
          <FavCard
            v-if="media.attr === 0 && media.type === 2"
            ref="favCardRefs"
            :data-key="media.id"
            :class="[
              'selectable border border-solid border-transparent',
              selectedIds.has(media.id) ? 'selected shadow-md' : 'hover:bg-gray-1',
            ]"
            :media="media"
            :download-episode="downloadEpisode"
            :checkbox-checked="checkboxChecked"
            :handle-checkbox-click="handleCheckboxClick"
            :handle-context-menu="handleContextMenu"
            :search="searchPaneRef?.search" />

          <FavCard
            v-else-if="media.attr === 0 && media.type === 24"
            ref="favCardRefs"
            class="border border-solid border-transparent hover:border-gray-3"
            :media="media"
            :search="searchPaneRef?.search" />

          <FavCard
            v-else
            ref="favCardRefs"
            class="border border-solid border-transparent hover:border-gray-3"
            :media="media" />
        </template>
      </div>
    </SelectionArea>

    <div class="flex gap-2 m-2 box-border">
      <n-pagination :page-count="pageCount" :page="currentPage" @update:page="getFav($event)" />
      <n-select class="w-40%" size="small" v-model:value="selectedMediaListId" :options="selectOptions" />
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
