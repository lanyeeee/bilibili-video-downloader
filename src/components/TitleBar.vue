<script setup lang="ts">
import icon from '../../src-tauri/icons/128x128.png'
import { PhCopySimple, PhMinus, PhSquare, PhX } from '@phosphor-icons/vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { onMounted, ref } from 'vue'
import { platform } from '@tauri-apps/plugin-os'

const appWindow = getCurrentWindow()
const windowMaximised = ref<boolean>(false)
const windowFullscreen = ref<boolean>(false)

const currentPlatform = platform()

const loginDialogShowing = ref<boolean>(false)

onMounted(async () => {
  windowMaximised.value = await appWindow.isMaximized()
  windowFullscreen.value = await appWindow.isFullscreen()

  await appWindow.onResized(async () => {
    windowMaximised.value = await appWindow.isMaximized()
    windowFullscreen.value = await appWindow.isFullscreen()
  })
})
</script>

<template>
  <div
    data-tauri-drag-region
    class="flex items-center bg-[#F2F2F2] border-b-solid border-b-1 border-b-[#DADADA] h-9 flex-shrink-0 select-none">
    <div v-if="currentPlatform === 'macos' && !windowFullscreen" class="ml-16" />
    <img data-tauri-drag-region :src="icon" alt="icon" class="ml-2 mr-2 w-6 h-6" draggable="false" />
    <span data-tauri-drag-region class="text-base select-none">哔哩哔哩视频下载器</span>

    <div class="ml-auto" />

    <div v-if="currentPlatform !== 'macos'" class="flex items-center select-none">
      <div
        class="flex items-center justify-center h-9 w-9 hover:bg-gray/20 cursor-pointer"
        @click="appWindow.minimize()">
        <PhMinus size="16" />
      </div>
      <div
        class="flex items-center justify-center h-9 w-9 hover:bg-gray/20 cursor-pointer"
        @click="appWindow.toggleMaximize()">
        <PhCopySimple v-if="windowMaximised" size="16" />
        <PhSquare v-else size="14" />
      </div>
      <div
        class="flex items-center justify-center h-9 w-9 hover:bg-red hover:text-white cursor-pointer"
        @click="appWindow.close()">
        <PhX size="16" />
      </div>
    </div>
    <LoginDialog v-model:showing="loginDialogShowing" />
  </div>
</template>
