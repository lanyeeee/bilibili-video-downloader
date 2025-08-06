<script setup lang="tsx">
import { onMounted, ref } from 'vue'
import { useStore } from './store.ts'
import LogDialog from './dialogs/LogDialog.vue'
import { PhClockCounterClockwise, PhInfo, PhGearSix } from '@phosphor-icons/vue'
import AboutDialog from './dialogs/AboutDialog.vue'
import { platform } from '@tauri-apps/plugin-os'
import TitleBar from './components/TitleBar.vue'
import SettingsDialog from './dialogs/SettingsDialog/SettingsDialog.vue'

const currentPlatform = platform()

const store = useStore()

const logDialogShowing = ref<boolean>(false)
const aboutDialogShowing = ref<boolean>(false)
const settingsDialogShowing = ref<boolean>(false)

onMounted(() => {
  // 屏蔽浏览器右键菜单
  document.oncontextmenu = (event) => {
    event.preventDefault()
  }
})
</script>

<template>
  <div
      :class="[
      'h-screen flex flex-col',
      {
        'box-border border border-solid border-gray-3': currentPlatform === 'linux',
      },
    ]">
    <TitleBar />

    <div v-if="store.config !== undefined" class="h-full w-full flex overflow-hidden select-none">
      <div class="flex flex-col box-border p-1 border-r-solid border-r-1 border-r-[#DADADA] bg-[#F9F9F9] flex-shrink-0">
        <n-tooltip placement="right" trigger="hover" :show-arrow="false">
          配置
          <template #trigger>
            <n-button text class="mt-auto py-1 px-2" @click="settingsDialogShowing = true">
              <n-icon size="28">
                <PhGearSix />
              </n-icon>
            </n-button>
          </template>
        </n-tooltip>

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

        <n-tooltip placement="right" trigger="hover" :show-arrow="false">
          关于
          <template #trigger>
            <n-button text class="py-1 px-2 mb-2" @click="aboutDialogShowing = true">
              <n-icon size="28">
                <PhInfo />
              </n-icon>
            </n-button>
          </template>
        </n-tooltip>
      </div>
    </div>

    <SettingsDialog v-model:showing="settingsDialogShowing" />
    <LogDialog v-model:showing="logDialogShowing" />
    <AboutDialog v-model:showing="aboutDialogShowing" />
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
