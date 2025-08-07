<script setup lang="ts">
defineProps<{
  checked: boolean
  onClick: () => void
}>()
</script>

<template>
  <div
    class="w-7 h-7 flex items-center justify-center rounded cursor-pointer border border-solid border-white"
    :class="checked ? 'bg-sky-5' : 'bg-gray/50'"
    @click="onClick">
    <svg
      v-show="checked"
      class="w-5 h-5 text-white"
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      stroke-width="4"
      stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
    </svg>
  </div>
</template>
