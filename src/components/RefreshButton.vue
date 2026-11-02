<script setup lang="ts">
import { ref } from 'vue'
import { NButton, NIcon, NTooltip } from 'naive-ui'
import { PhArrowClockwise } from '@phosphor-icons/vue'

const props = defineProps<{
  refresh: () => Promise<void> | void
}>()

const refreshing = ref<boolean>(false)

async function handleClick() {
  if (refreshing.value) {
    return
  }
  refreshing.value = true
  try {
    await props.refresh()
  } finally {
    refreshing.value = false
  }
}
</script>

<template>
  <n-tooltip trigger="hover" :show-arrow="false">
    <template #trigger>
      <n-button size="small" :loading="refreshing" @click="handleClick">
        <template #icon>
          <n-icon size="16">
            <PhArrowClockwise weight="bold" />
          </n-icon>
        </template>
      </n-button>
    </template>
    刷新
  </n-tooltip>
</template>
