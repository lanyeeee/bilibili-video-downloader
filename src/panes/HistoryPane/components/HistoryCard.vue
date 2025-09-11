<script setup lang="ts">
import { EpisodeType, HistoryDetail } from '../../../bindings.ts'
import { SearchType } from '../../SearchPane/SearchPane.vue'
import { computed, inject, ref } from 'vue'
import { navDownloadButtonRefKey } from '../../../injection_keys.ts'
import { ensureHttps, isElementInViewport, playTaskToQueueAnimation } from '../../../utils.tsx'
import { PhDownloadSimple, PhGoogleChromeLogo, PhMagnifyingGlass } from '@phosphor-icons/vue'
import SimpleCheckbox from '../../../components/SimpleCheckbox.vue'
import IconButton from '../../../components/IconButton.vue'

const props = defineProps<{
  episodeType: EpisodeType
  historyDetail: HistoryDetail
  downloadEpisode?: (historyDetail: HistoryDetail) => Promise<void>
  checkboxChecked?: (historyDetail: HistoryDetail) => boolean
  handleCheckboxClick?: (historyDetail: HistoryDetail) => void
  handleContextMenu?: (historyDetail: HistoryDetail) => void
  search?: (input: string, searchType: SearchType) => void
}>()

const navDownloadButtonRef = inject(navDownloadButtonRefKey)
const rootDivRef = ref<HTMLDivElement>()
const downloadButtonRef = ref<InstanceType<typeof IconButton>>()

const openInBrowserHref = computed<string | undefined>(() => {
  if (props.episodeType === 'Normal') {
    return `https://www.bilibili.com/video/${props.historyDetail.history.bvid}/`
  } else if (props.episodeType === 'Bangumi') {
    return `https://www.bilibili.com/bangumi/play/ep${props.historyDetail.history.epid}`
  } else if (props.episodeType === 'Cheese') {
    return `https://www.bilibili.com/cheese/play/ep${props.historyDetail.history.epid}`
  }
  return undefined
})

const downloadHint = computed<string | undefined>(() => {
  if (props.episodeType !== 'Normal') {
    return '下载请点击左下角放大镜按钮'
  }
  return undefined
})

const subTitle = computed<string>(() => {
  if (props.episodeType === 'Normal') {
    return ''
  } else if (props.historyDetail.long_title !== '') {
    return props.historyDetail.long_title
  } else if (props.historyDetail.show_title !== '') {
    return props.historyDetail.show_title
  } else if (props.historyDetail.new_desc !== '') {
    return props.historyDetail.new_desc
  }
  return ''
})

const timeFormat = computed(() => {
  function isToday(date: Date): boolean {
    const today = new Date()
    return (
      date.getDate() === today.getDate() &&
      date.getMonth() === today.getMonth() &&
      date.getFullYear() === today.getFullYear()
    )
  }

  function isYesterday(date: Date): boolean {
    const yesterday = new Date()
    yesterday.setDate(yesterday.getDate() - 1)
    return (
      date.getDate() === yesterday.getDate() &&
      date.getMonth() === yesterday.getMonth() &&
      date.getFullYear() === yesterday.getFullYear()
    )
  }

  const date = new Date(props.historyDetail.view_at * 1000)

  if (isToday(date)) {
    return "'今天' HH:mm"
  }
  if (isYesterday(date)) {
    return "'昨天' HH:mm"
  } else {
    return 'MM-dd HH:mm'
  }
})

async function handleDownloadClick() {
  if (props.downloadEpisode === undefined) {
    return
  }

  await props.downloadEpisode(props.historyDetail)
  playDownloadAnimation()
}

function playDownloadAnimation() {
  if (rootDivRef.value === undefined) {
    return
  }

  const from = downloadButtonRef.value?.$el
  const to = navDownloadButtonRef?.value

  if (from instanceof Element && to !== undefined) {
    if (isElementInViewport(rootDivRef.value)) {
      // 只有卡片在视口内才播放动画
      playTaskToQueueAnimation(from, to)
    }
  }
}

function searchInSearchPane() {
  if (props.search === undefined) {
    return
  }

  if (props.episodeType === 'Normal') {
    props.search(props.historyDetail.history.bvid, 'Normal')
  } else if (props.episodeType === 'Bangumi') {
    props.search(`ep${props.historyDetail.history.epid}`, 'Bangumi')
  } else if (props.episodeType === 'Cheese') {
    props.search(`ep${props.historyDetail.history.epid}`, 'Cheese')
  }
}

defineExpose({ playDownloadAnimation, historyDetail: props.historyDetail })
</script>

<template>
  <div
    class="flex flex-col w-200px relative p-3 rounded-lg"
    ref="rootDivRef"
    :title="downloadHint"
    @contextmenu="handleContextMenu?.(historyDetail)">
    <SimpleCheckbox
      v-if="episodeType === 'Normal' && handleCheckboxClick !== undefined && checkboxChecked !== undefined"
      class="absolute top-6 left-6 z-1 backdrop-blur-2"
      :checked="checkboxChecked(historyDetail)"
      :on-click="() => handleCheckboxClick?.(historyDetail)" />
    <div v-if="historyDetail.badge !== ''" class="absolute top-6 right-6 z-1 bg-[#ff6699] text-white px-1 rounded">
      {{ historyDetail.badge }}
    </div>
    <img
      class="w-200px h-125px rounded-lg object-cover lazyload"
      :data-src="`${ensureHttps(historyDetail.cover)}@672w_378h_1c.webp`"
      :key="historyDetail.cover"
      alt=""
      draggable="false" />

    <div class="w-full flex flex-col h-45px mt-2">
      <span class="line-clamp-2" :title="historyDetail.title">{{ historyDetail.title }}</span>
    </div>

    <div class="flex items-center whitespace-nowrap text-gray text-12px w-full overflow-hidden">
      <a
        v-if="episodeType === 'Normal'"
        class="min-w-0 color-inherit no-underline hover:text-sky-5 mr-1"
        :href="`https://space.bilibili.com/${historyDetail.author_mid}`"
        target="_blank"
        draggable="false">
        <div class="truncate text-ellipsis" :title="historyDetail.author_name">{{ historyDetail.author_name }}</div>
      </a>
      <a
        v-else
        class="min-w-0 color-inherit no-underline hover:text-sky-5 mr-1"
        :href="openInBrowserHref"
        target="_blank"
        draggable="false">
        <div class="truncate text-ellipsis" :title="subTitle">{{ subTitle }}</div>
      </a>

      <span class="ml-auto flex-shrink-0" title="上次观看时间">
        <n-time unix :format="timeFormat" :time="historyDetail.view_at" />
      </span>
    </div>

    <div class="flex gap-1 items-center">
      <IconButton title="在浏览器中打开" :href="openInBrowserHref">
        <PhGoogleChromeLogo :size="24" />
      </IconButton>
      <IconButton v-if="search !== undefined" title="在下载器内搜索" @click="searchInSearchPane">
        <PhMagnifyingGlass :size="24" />
      </IconButton>
      <IconButton
        v-if="props.downloadEpisode !== undefined"
        ref="downloadButtonRef"
        class="ml-auto"
        title="一键下载"
        @click="handleDownloadClick">
        <PhDownloadSimple :size="24" />
      </IconButton>
    </div>
  </div>
</template>
