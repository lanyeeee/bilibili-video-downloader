<script setup lang="ts">
import { commands, NormalSearchResult } from '../../../bindings.ts'
import CollectionCard from './CollectionCard.vue'
import EpisodeCard from './EpisodeCard.vue'

const props = defineProps<{
  normalResult: NormalSearchResult
}>()

async function downloadEpisode() {
  await commands.createDownloadTasks({
    Normal: {
      info: props.normalResult,
      aid_cid_pairs: [[props.normalResult.aid, null]],
    },
  })
}
</script>

<template>
  <div class="flex flex-col h-full gap-2 select-none">
    <div class="px-2 mt-2">
      <EpisodeCard
        class="border border-solid border-gray-2"
        :search-result="normalResult"
        :episode="normalResult"
        :episode-type="'NormalSingle'"
        :download-episode="downloadEpisode" />
    </div>
    <CollectionCard
      class="mt-auto"
      :title="normalResult.title"
      :description="normalResult.desc"
      :cover="normalResult.pic"
      :up-name="normalResult.owner.name"
      :up-avatar="normalResult.owner.face"
      :up-uid="normalResult.owner.mid" />
  </div>
</template>
