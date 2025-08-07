<script setup lang="ts">
import ColorfulTag from '../../../components/ColorfulTag.vue'
import { commands, NormalInfo, PageInNormalEp, PageInNormal } from '../../../bindings.ts'
import { EpisodeInfo } from './EpisodeCard.vue'
import { ensureHttps, playTaskToQueueAnimation } from '../../../utils.tsx'

const props = defineProps<{
  info: NormalInfo
  pages: PageInNormalEp[] | PageInNormal[]
  episodeInfo: EpisodeInfo
  downloadButtonRef?: HTMLDivElement
}>()

function handleDownloadClick(aid: number, cid: number, event: MouseEvent) {
  const from = event.currentTarget
  const to = props.downloadButtonRef

  commands.createDownloadTasks({ Normal: { info: props.info, aid_cid_pairs: [[aid, cid]] } })

  if (from instanceof Element && to !== undefined) {
    playTaskToQueueAnimation(from, to)
  }
}
</script>

<template>
  <div v-if="pages !== undefined" class="flex flex-col max-h-60vh overflow-auto">
    <div class="flex flex-shrink-0 items-start">
      <img
        class="w-200px h-125px rounded-lg object-cover lazyload"
        :data-src="`${ensureHttps(episodeInfo.cover)}@672w_378h_1c.webp`"
        :key="episodeInfo.cover"
        alt=""
        draggable="false" />
      <div class="ml-2 font-bold text-base line-clamp-5">{{ episodeInfo.title }}</div>
    </div>
    <div class="flex flex-col gap-1 overflow-auto mt-2">
      <div v-for="page in pages" class="whitespace-nowrap flex" :key="page.cid">
        <ColorfulTag class="bg-violet-2 truncate" :title="page.part" color="violet">
          P{{ page.page }} {{ page.part }}
        </ColorfulTag>
        <n-button
          class="ml-2"
          size="tiny"
          type="primary"
          @click="handleDownloadClick(episodeInfo.aid, page.cid, $event)">
          下载该分P
        </n-button>
      </div>
    </div>
  </div>
</template>
