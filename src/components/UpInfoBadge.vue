<script setup lang="ts">
import { ensureHttps } from '../utils.tsx'

defineProps<{
  upName: string
  upAvatar: string
  upUid: number
}>()
</script>

<template>
  <a
    class="flex items-center color-inherit no-underline"
    :href="`https://space.bilibili.com/${upUid}`"
    target="_blank"
    draggable="false">
    <img class="w-8 h-8 rounded-full" :src="`${ensureHttps(upAvatar)}@128w_128h`" alt="" draggable="false" />
    <div class="line-clamp-1" :title="upName">{{ upName }}</div>
  </a>
</template>
