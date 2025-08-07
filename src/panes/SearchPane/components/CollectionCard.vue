<script setup lang="ts">
import UpInfoBadge from '../../../components/UpInfoBadge.vue'
import { ensureHttps } from '../../../utils.tsx'

defineProps<{
  title: string
  description?: string
  cover: string
  upName?: string
  upAvatar?: string
  upUid?: number
}>()
</script>

<template>
  <div class="flex m-2 rounded-lg">
    <img class="object-contain mr-2 rounded-lg h-25" :src="`${ensureHttps(cover)}@450w`" alt="" draggable="false" />
    <div class="flex flex-col w-full break-words overflow-auto">
      <span class="font-bold text-lg line-clamp-1">{{ title }}</span>
      <span class="line-clamp-2">{{ description }}</span>
      <UpInfoBadge
        v-if="upName !== undefined && upAvatar !== undefined && upUid !== undefined"
        class="mt-auto ml-auto"
        :upName="upName"
        :upAvatar="upAvatar"
        :upUid="upUid" />
    </div>
  </div>
</template>
