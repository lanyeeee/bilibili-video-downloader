<script setup lang="ts">
import { MediaInFav } from '../../../bindings.ts'
import { PhDownloadSimple, PhGoogleChromeLogo, PhMagnifyingGlass } from '@phosphor-icons/vue'
import SimpleCheckbox from '../../../components/SimpleCheckbox.vue'
import { ensureHttps, isElementInViewport, playTaskToQueueAnimation } from '../../../utils.tsx'
import { computed, inject, ref } from 'vue'
import { navDownloadButtonRefKey } from '../../../injection_keys.ts'
import { SearchType } from '../../SearchPane/SearchPane.vue'

const props = defineProps<{
  media: MediaInFav
  downloadEpisode?: (media: MediaInFav) => Promise<void>
  checkboxChecked?: (media: MediaInFav) => boolean
  handleCheckboxClick?: (media: MediaInFav) => void
  handleContextMenu?: (media: MediaInFav) => void
  search?: (input: string, searchType: SearchType) => void
}>()

const navDownloadButtonRef = inject(navDownloadButtonRefKey)
const rootDivRef = ref<HTMLDivElement>()
const downloadButtonRef = ref<HTMLDivElement>()

const openInBrowserHref = computed<string | undefined>(() => {
  if (props.media.type === 2) {
    return `https://www.bilibili.com/video/${props.media.bvid}/`
  } else if (props.media.type === 12) {
    return `https://www.bilibili.com/audio/au${props.media.id}`
  } else if (props.media.type === 24) {
    return `https://www.bilibili.com/bangumi/play/ep${props.media.id}`
  }
  return undefined
})

const hint = computed<string | undefined>(() => {
  if (props.media.type === 12) {
    return '不支持音乐下载'
  } else if (props.media.type === 24) {
    return '下载请点击左下角放大镜按钮'
  }
  return undefined
})

async function handleDownloadClick() {
  if (props.downloadEpisode === undefined) {
    return
  }

  await props.downloadEpisode(props.media)
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

function searchInSearchPane() {
  if (props.search === undefined) {
    return
  }

  if (props.media.type === 2) {
    props.search(props.media.bvid, 'Normal')
  } else if (props.media.type === 24) {
    props.search(`ep${props.media.id}`, 'Bangumi')
  }
}

defineExpose({ playDownloadAnimation, media: props.media })
</script>

<template>
  <div
    class="flex flex-col w-200px relative p-3 rounded-lg"
    ref="rootDivRef"
    :title="hint"
    @contextmenu="handleContextMenu?.(media)">
    <SimpleCheckbox
      v-if="media.attr === 0 && handleCheckboxClick !== undefined && checkboxChecked !== undefined"
      class="absolute top-6 left-6 z-1 backdrop-blur-2"
      :checked="checkboxChecked(media)"
      :on-click="() => handleCheckboxClick?.(media)" />
    <div v-if="media.type === 24" class="absolute top-6 right-6 z-1 bg-[#ff6699] text-white px-1 rounded">番剧</div>
    <div v-else-if="media.type === 12" class="absolute top-6 right-6 z-1 bg-green-5 text-white px-1 rounded">音乐</div>
    <img
      class="w-200px h-125px rounded-lg object-cover lazyload"
      :data-src="`${ensureHttps(media.cover)}@672w_378h_1c.webp`"
      :key="media.cover"
      alt=""
      draggable="false" />

    <div class="w-full flex flex-col h-45px mt-2">
      <span class="line-clamp-2" :title="media.title">{{ media.title }}</span>
    </div>

    <div class="flex items-center whitespace-nowrap text-gray text-12px w-full overflow-hidden">
      <a
        v-if="media.type !== 24"
        class="min-w-0 color-inherit no-underline hover:text-sky-5 mr-1"
        :href="`https://space.bilibili.com/${media.upper.mid}`"
        target="_blank"
        draggable="false">
        <div class="truncate text-ellipsis" :title="media.upper.name">{{ media.upper.name }}</div>
      </a>
      <div v-else class="truncate text-ellipsis">{{ media.intro }}</div>

      <span class="ml-auto flex-shrink-0" title="收藏时间">
        <n-time unix type="date" :time="media.fav_time" />
      </span>
    </div>

    <div class="flex gap-1 items-center">
      <a
        :href="openInBrowserHref"
        target="_blank"
        draggable="false"
        title="在浏览器中打开"
        class="p-1 rounded-lg flex items-center justify-between text-gray-6 hover:bg-sky-5 hover:text-white active:bg-sky-6">
        <PhGoogleChromeLogo :size="24" />
      </a>

      <div
        v-if="search !== undefined"
        title="在下载器内搜索"
        class="cursor-pointer p-1 rounded-lg flex items-center justify-between text-gray-6 hover:bg-sky-5 hover:text-white active:bg-sky-6"
        @click="searchInSearchPane">
        <PhMagnifyingGlass :size="24" />
      </div>

      <div
        v-if="props.downloadEpisode !== undefined"
        ref="downloadButtonRef"
        title="一键下载"
        class="ml-auto cursor-pointer p-1 rounded-lg flex items-center justify-between text-gray-6 hover:bg-sky-5 hover:text-white active:bg-sky-6"
        @click="handleDownloadClick">
        <PhDownloadSimple :size="24" />
      </div>
    </div>
  </div>
</template>
