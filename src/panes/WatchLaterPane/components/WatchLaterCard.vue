<script setup lang="ts">
import { MediaInWatchLater } from '../../../bindings.ts'
import { computed, inject, ref } from 'vue'
import { navDownloadButtonRefKey } from '../../../injection_keys.ts'
import { ensureHttps, extractEpId, isElementInViewport, playTaskToQueueAnimation } from '../../../utils.tsx'
import { PhDownloadSimple, PhGoogleChromeLogo, PhMagnifyingGlass } from '@phosphor-icons/vue'
import SimpleCheckbox from '../../../components/SimpleCheckbox.vue'
import { SearchType } from '../../SearchPane/SearchPane.vue'
import IconButton from '../../../components/IconButton.vue'

const props = defineProps<{
  media: MediaInWatchLater
  downloadEpisode?: (media: MediaInWatchLater) => Promise<void>
  checkboxChecked?: (media: MediaInWatchLater) => boolean
  handleCheckboxClick?: (media: MediaInWatchLater) => void
  handleContextMenu?: (media: MediaInWatchLater) => void
  search?: (input: string, searchType: SearchType) => void
}>()

const navDownloadButtonRef = inject(navDownloadButtonRefKey)
const rootDivRef = ref<HTMLDivElement>()
const downloadButtonRef = ref<InstanceType<typeof IconButton>>()

const openInBrowserHref = computed<string>(() => {
  if (props.media.redirect_url === null) {
    return `https://www.bilibili.com/video/${props.media.bvid}/`
  } else {
    const epId = extractEpId(props.media.redirect_url)
    return `https://www.bilibili.com/bangumi/play/ep${epId}`
  }
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

  if (props.media.redirect_url === null) {
    props.search(props.media.bvid, 'Normal')
  } else {
    const epId = extractEpId(props.media.redirect_url)
    props.search(`ep${epId}`, 'Bangumi')
  }
}

defineExpose({ playDownloadAnimation, media: props.media })
</script>

<template>
  <div
    class="flex flex-col w-200px relative p-3 rounded-lg"
    @contextmenu="handleContextMenu?.(media)"
    :title="media.redirect_url !== null ? '下载请点击左下角放大镜按钮' : undefined"
    ref="rootDivRef">
    <SimpleCheckbox
      v-if="media.redirect_url === null && handleCheckboxClick !== undefined && checkboxChecked !== undefined"
      class="absolute top-6 left-6 z-1 backdrop-blur-2"
      :checked="checkboxChecked(media)"
      :on-click="() => handleCheckboxClick?.(media)" />
    <div v-if="media.redirect_url !== null" class="absolute top-6 right-6 z-1 bg-[#ff6699] text-white px-1 rounded">
      番剧
    </div>
    <img
      class="w-200px h-125px rounded-lg object-cover lazyload"
      :data-src="`${ensureHttps(media.pic)}@672w_378h_1c.webp`"
      :key="media.pic"
      alt=""
      draggable="false" />

    <div class="w-full flex flex-col h-45px mt-2">
      <span class="line-clamp-2" :title="media.title">{{ media.title }}</span>
    </div>

    <div class="flex items-center whitespace-nowrap text-gray text-12px w-full overflow-hidden">
      <a
        class="min-w-0 color-inherit no-underline hover:text-sky-5 mr-1"
        :href="`https://space.bilibili.com/${media.owner.mid}`"
        target="_blank"
        draggable="false">
        <div class="truncate text-ellipsis" :title="media.owner.name">{{ media.owner.name }}</div>
      </a>
      <span class="ml-auto flex-shrink-0" title="发布时间">
        <n-time unix type="date" :time="media.pubdate" />
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
        v-if="downloadEpisode !== undefined"
        ref="downloadButtonRef"
        title="一键下载"
        class="ml-auto"
        @click="handleDownloadClick">
        <PhDownloadSimple :size="24" />
      </IconButton>
    </div>
  </div>
</template>
