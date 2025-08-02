<script setup lang="tsx">
import { onMounted, ref } from 'vue'
import { useStore } from './store.ts'
import LogDialog from './dialogs/LogDialog.vue'
import { PhClockCounterClockwise } from '@phosphor-icons/vue'
import { commands } from './bindings.ts'

const store = useStore()

const logDialogShowing = ref<boolean>(false)

onMounted(async () => {
  // 屏蔽浏览器右键菜单
  document.oncontextmenu = (event) => {
    event.preventDefault()
  }

  store.config = await commands.getConfig()
})
</script>

<template>
  <div class="h-screen flex flex-col">
    <div v-if="store.config !== undefined" class="h-full w-full flex overflow-hidden select-none">
      <div class="flex flex-col box-border p-1 border-r-solid border-r-1 border-r-[#DADADA] bg-[#F9F9F9] flex-shrink-0">
        <n-tooltip placement="right" trigger="hover" :show-arrow="false">
          日志
          <template #trigger>
            <n-button text class="py-1 px-2" @click="logDialogShowing = true">
              <n-icon size="28">
                <PhClockCounterClockwise />
              </n-icon>
            </n-button>
          </template>
        </n-tooltip>
      </div>
    </div>

    <LogDialog v-model:showing="logDialogShowing" />
  </div>
</template>

<style scoped>
:global(.n-notification-main__header) {
  @apply break-words;
}

:global(.n-tabs-pane-wrapper) {
  @apply h-full;
}

:global(.selection-area) {
  @apply bg-[rgba(46,115,252,0.5)];
}

:deep(.n-badge-sup) {
  @apply pointer-events-none;
}
</style>
