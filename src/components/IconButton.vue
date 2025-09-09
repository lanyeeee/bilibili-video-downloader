<script setup lang="ts">
defineProps<{
  href?: string
}>()
</script>

<template>
  <a
    v-if="href !== undefined"
    :href="href"
    target="_blank"
    draggable="false"
    class="cursor-pointer p-1 rounded-lg flex items-center justify-between text-gray-6 hover:bg-sky-5 hover:text-white active:bg-sky-6 active:text-white">
    <slot />
  </a>
  <div
    v-else
    class="cursor-pointer p-1 rounded-lg flex items-center justify-between text-gray-6 hover:bg-sky-5 hover:text-white active:bg-sky-6 active:text-white">
    <slot />
  </div>
</template>
