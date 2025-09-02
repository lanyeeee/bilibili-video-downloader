<script setup lang="ts">
const { size = 'medium' } = defineProps<{
  size?: 'small' | 'medium'
  checked: boolean
  onClick: () => void
}>()
</script>

<template>
  <div
    class="flex items-center justify-center rounded cursor-pointer border border-solid border-white"
    :class="{
      'bg-sky-5': checked,
      'bg-gray/50': !checked,
      'w-7 h-7': size === 'medium',
      'w-5 h-5': size === 'small',
    }"
    @click="onClick">
    <svg
      v-show="checked"
      class="text-white"
      :class="{
        'w-5 h-5': size === 'medium',
        'w-4 h-4': size === 'small',
      }"
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      stroke-width="4"
      stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
    </svg>
  </div>
</template>
