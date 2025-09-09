<script setup lang="ts">
import { inject, ref } from 'vue'
import { navDownloadButtonRefKey, searchPaneRefKey } from '../../../injection_keys.ts'
import { ensureHttps, isElementInViewport, playTaskToQueueAnimation } from '../../../utils.tsx'
import { PhDownloadSimple, PhGoogleChromeLogo, PhMagnifyingGlass } from '@phosphor-icons/vue'
import { EpInBangumiFollow } from '../../../bindings.ts'
import SimpleCheckbox from '../../../components/SimpleCheckbox.vue'
import IconButton from '../../../components/IconButton.vue'

const searchPaneRef = inject(searchPaneRefKey)

const props = defineProps<{
  ep: EpInBangumiFollow
  downloadEpisode: (ep: EpInBangumiFollow) => Promise<void>
  checkboxChecked: (ep: EpInBangumiFollow) => boolean
  handleCheckboxClick: (ep: EpInBangumiFollow) => void
  handleContextMenu: (ep: EpInBangumiFollow) => void
}>()

const navDownloadButtonRef = inject(navDownloadButtonRefKey)
const rootDivRef = ref<HTMLDivElement>()
const downloadButtonRef = ref<InstanceType<typeof IconButton>>()

async function handleDownloadClick() {
  if (props.downloadEpisode === undefined) {
    return
  }

  await props.downloadEpisode(props.ep)
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

defineExpose({ playDownloadAnimation, ep: props.ep })
</script>

<template>
  <div class="flex flex-col w-200px relative p-3 rounded-lg" ref="rootDivRef" @contextmenu="handleContextMenu(ep)">
    <SimpleCheckbox
      class="absolute top-5 left-5 z-1 backdrop-blur-2"
      size="small"
      :checked="checkboxChecked(ep)"
      :on-click="() => handleCheckboxClick(ep)" />
    <div class="flex">
      <img
        class="w-90px h-120px rounded-lg object-cover lazyload"
        :data-src="`${ensureHttps(ep.cover)}@308w_410h_1c.webp`"
        :key="ep.cover"
        alt=""
        draggable="false" />
      <div class="flex flex-col ml-1">
        <span class="line-clamp-2" :title="ep.title">{{ ep.title }}</span>
        <span class="text-gray mt-auto">{{ ep.season_type_name }} · {{ ep.areas[0].name }}</span>
        <span class="text-gray">{{ ep.new_ep.index_show }}</span>
        <span class="text-gray">{{ ep.progress }}</span>
      </div>
    </div>

    <div class="flex gap-1 items-center mt-2">
      <IconButton title="在浏览器中打开" :href="`https://www.bilibili.com/bangumi/play/ss${ep.season_id}`">
        <PhGoogleChromeLogo :size="24" />
      </IconButton>
      <IconButton title="在下载器内搜索" @click="searchPaneRef?.search(`ss${props.ep.season_id}`, 'Bangumi')">
        <PhMagnifyingGlass :size="24" />
      </IconButton>
      <IconButton
        v-if="downloadEpisode !== undefined"
        ref="downloadButtonRef"
        class="ml-auto"
        title="一键下载"
        @click="handleDownloadClick">
        <PhDownloadSimple :size="24" />
      </IconButton>
    </div>
  </div>
</template>
