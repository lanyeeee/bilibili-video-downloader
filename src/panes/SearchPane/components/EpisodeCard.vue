<script setup lang="tsx">
import { computed, inject, onMounted, onUpdated, ref } from 'vue'
import { BangumiSearchResult, EpInBangumi, EpInNormal, NormalInfo, NormalSearchResult } from '../../../bindings.ts'
import SimpleCheckbox from '../../../components/SimpleCheckbox.vue'
import { PhDownloadSimple, PhGoogleChromeLogo, PhQueue, PhMagnifyingGlass } from '@phosphor-icons/vue'
import { useDialog } from 'naive-ui'
import PartsDialogContent from './PartsDialogContent.vue'
import { ensureHttps, extractBvid, isElementInViewport, playTaskToQueueAnimation } from '../../../utils.tsx'
import { navDownloadButtonRefKey } from '../../../injection_keys.ts'
import { SearchType } from '../SearchPane.vue'

onMounted(() => console.log('EpisodeCard mounted'))
onUpdated(() => console.log('EpisodeCard updated'))

const dialog = useDialog()

export type EpisodeInfo = {
  episodeType: 'Normal' | 'Bangumi' | 'Cheese'
  aid: number
  bvid?: string
  epId?: number
  href?: string
  cover: string
  title: string
  upName: string
  upUid: number
  pubTime: number
  favTime?: number
}

const props = defineProps<{
  searchResult: NormalSearchResult | BangumiSearchResult
  episode: NormalInfo | EpInNormal | EpInBangumi
  episodeType: 'NormalSingle' | 'NormalSeason' | 'Bangumi'
  downloadEpisode?: (episodeInfo: EpisodeInfo) => Promise<void>
  checkboxChecked?: (episodeInfo: EpisodeInfo) => boolean
  handleCheckboxClick?: (episodeInfo: EpisodeInfo) => void
  handleContextMenu?: (episodeInfo: EpisodeInfo) => void
  search?: (input: string, searchType: SearchType) => void
}>()

const navDownloadButtonRef = inject(navDownloadButtonRefKey)
const rootDivRef = ref<HTMLDivElement>()
const downloadButtonRef = ref<HTMLDivElement>()

const episodeInfo = computed<EpisodeInfo>(() => {
  if (props.episodeType === 'NormalSingle') {
    const episode = props.episode as NormalInfo
    return {
      episodeType: 'Normal',
      aid: episode.aid,
      bvid: episode.bvid,
      href: `https://www.bilibili.com/video/${episode.bvid}/`,
      cover: episode.pic,
      title: episode.title,
      upName: episode.owner.name,
      upUid: episode.owner.mid,
      pubTime: episode.pubdate,
    }
  } else if (props.episodeType === 'NormalSeason') {
    const episode = props.episode as EpInNormal
    return {
      episodeType: 'Normal',
      aid: episode.aid,
      bvid: episode.bvid,
      href: `https://www.bilibili.com/video/${episode.bvid}/`,
      cover: episode.arc.pic,
      title: episode.arc.title,
      upName: episode.arc.author.name,
      upUid: episode.arc.author.mid,
      pubTime: episode.arc.pubdate,
    }
  } else if (props.episodeType === 'Bangumi') {
    const episode = props.episode as EpInBangumi
    const searchResult = props.searchResult as BangumiSearchResult
    return {
      episodeType: 'Bangumi',
      aid: episode.aid,
      bvid: episode.bvid ?? undefined,
      epId: episode.ep_id,
      href:
        episode.link_type === null
          ? `https://www.bilibili.com/bangumi/play/ep${episode.ep_id}`
          : `https://www.bilibili.com/video/${extractBvid(episode.link)}/`,
      cover: episode.cover,
      title: episode.show_title ?? episode.title,
      upName: searchResult.info.up_info?.uname ?? '无',
      upUid: searchResult.info.up_info?.mid ?? 0,
      pubTime: episode.pub_time,
    }
  }
  throw new Error(`错误的 episodeType: ${props.episodeType}`)
})

const partsButtonShowing = computed<boolean>(() => {
  if (props.episodeType !== 'NormalSingle' && props.episodeType !== 'NormalSeason') {
    return false
  }
  const episode = props.episode as NormalInfo | EpInNormal
  return episode.pages.length > 1
})

function handlePartsButtonClick() {
  if (props.episodeType !== 'NormalSingle' && props.episodeType !== 'NormalSeason') {
    return
  }

  const episode = props.episode as NormalInfo | EpInNormal
  const info = props.searchResult as NormalSearchResult

  dialog.create({
    title: '分P',
    showIcon: false,
    content: () => (
      <PartsDialogContent
        info={info}
        pages={episode.pages}
        episodeInfo={episodeInfo.value}
        downloadButtonRef={navDownloadButtonRef?.value}
      />
    ),
  })
}

async function handleDownloadClick() {
  if (props.downloadEpisode === undefined) {
    return
  }

  await props.downloadEpisode(episodeInfo.value)

  playDownloadAnimation()
}

function playDownloadAnimation() {
  if (rootDivRef.value === undefined) {
    return
  }

  const from = downloadButtonRef.value
  const to = navDownloadButtonRef?.value

  if (from instanceof Element && to !== undefined) {
    if (isElementInViewport(rootDivRef.value)) {
      // 只有卡片在视口内才播放动画
      playTaskToQueueAnimation(from, to)
    }
  }
}

defineExpose({ playDownloadAnimation, episodeInfo })
</script>

<template>
  <div
    class="flex flex-col w-200px relative p-3 rounded-lg"
    @contextmenu="() => handleContextMenu?.(episodeInfo)"
    ref="rootDivRef">
    <SimpleCheckbox
      v-if="handleCheckboxClick !== undefined && checkboxChecked !== undefined"
      class="absolute top-6 left-6 z-1 backdrop-blur-2"
      :checked="checkboxChecked(episodeInfo)"
      :on-click="() => handleCheckboxClick?.(episodeInfo)" />
    <img
      class="w-200px h-125px rounded-lg object-cover lazyload"
      :data-src="`${ensureHttps(episodeInfo.cover)}@672w_378h_1c.webp`"
      :key="episodeInfo.cover"
      alt=""
      draggable="false" />

    <div class="w-full flex flex-col h-45px mt-2">
      <span class="line-clamp-2" :title="episodeInfo.title">{{ episodeInfo.title }}</span>
    </div>

    <div class="flex items-center whitespace-nowrap text-gray text-12px w-full overflow-hidden">
      <a
        class="min-w-0 color-inherit no-underline hover:text-sky-5 mr-1"
        :href="`https://space.bilibili.com/${episodeInfo.upUid}`"
        target="_blank"
        draggable="false">
        <div class="truncate text-ellipsis" :title="episodeInfo.upName">{{ episodeInfo.upName }}</div>
      </a>
      <span v-if="episodeInfo.favTime !== undefined" class="ml-auto flex-shrink-0" title="收藏时间">
        <n-time unix type="date" :time="episodeInfo.favTime" />
      </span>
      <span v-else-if="episodeInfo.pubTime !== 0" class="ml-auto flex-shrink-0" title="发布时间">
        <n-time unix type="date" :time="episodeInfo.pubTime" />
      </span>
    </div>

    <div class="flex gap-1 items-center">
      <a
        v-if="episodeInfo.href !== undefined"
        :href="episodeInfo.href"
        target="_blank"
        draggable="false"
        title="在浏览器中打开"
        class="p-1 rounded-lg flex items-center justify-between text-gray-6 hover:bg-sky-5 hover:text-white active:bg-sky-6">
        <PhGoogleChromeLogo :size="24" />
      </a>
      <div
        v-if="partsButtonShowing"
        title="查看分P"
        class="cursor-pointer p-1 rounded-lg flex items-center justify-between text-gray-6 hover:bg-sky-5 hover:text-white active:bg-sky-6"
        @click="handlePartsButtonClick">
        <PhQueue :size="24" />
      </div>
      <div
        v-if="search !== undefined && episodeInfo.bvid !== undefined"
        title="在下载器内搜索"
        class="cursor-pointer p-1 rounded-lg flex items-center justify-between text-gray-6 hover:bg-sky-5 hover:text-white active:bg-sky-6"
        @click="search(episodeInfo.bvid, 'Normal')">
        <PhMagnifyingGlass :size="24" />
      </div>

      <div
        ref="downloadButtonRef"
        v-if="downloadEpisode !== undefined"
        title="一键下载"
        class="ml-auto cursor-pointer p-1 rounded-lg flex items-center justify-between text-gray-6 hover:bg-sky-5 hover:text-white active:bg-sky-6"
        @click="handleDownloadClick">
        <PhDownloadSimple :size="24" />
      </div>
    </div>
  </div>
</template>
