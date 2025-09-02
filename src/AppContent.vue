<script setup lang="tsx">
import { onMounted, ref, provide } from 'vue'
import { useStore } from './store.ts'
import LogDialog from './dialogs/LogDialog.vue'
import {
  PhClockCounterClockwise,
  PhInfo,
  PhGearSix,
  PhMagnifyingGlass,
  PhStar,
  PhClock,
  PhHeart,
  PhDownload,
} from '@phosphor-icons/vue'
import AboutDialog from './dialogs/AboutDialog.vue'
import { platform } from '@tauri-apps/plugin-os'
import TitleBar from './components/TitleBar.vue'
import SettingsDialog from './dialogs/SettingsDialog/SettingsDialog.vue'
import SearchPane from './panes/SearchPane/SearchPane.vue'
import FavPane from './panes/FavPane/FavPane.vue'
import WatchLaterPane from './panes/WatchLaterPane/WatchLaterPane.vue'
import DownloadPane from './panes/DownloadPane/DownloadPane.vue'
import { searchPaneRefKey, navDownloadButtonRefKey } from './injection_keys.ts'
import BangumiFollowPane from './panes/BangumiFollow/BangumiFollowPane.vue'

export type CurrentNavName = 'search' | 'fav' | 'watch_later' | 'bangumi_follow' | 'download'

const currentPlatform = platform()

const store = useStore()

const logDialogShowing = ref<boolean>(false)
const aboutDialogShowing = ref<boolean>(false)
const settingsDialogShowing = ref<boolean>(false)

const searchPaneRef = ref<InstanceType<typeof SearchPane>>()
const downloadButtonRef = ref<HTMLDivElement>()

provide(searchPaneRefKey, searchPaneRef)
provide(navDownloadButtonRefKey, downloadButtonRef)

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
          搜索
          <template #trigger>
            <div
              class="flex cursor-pointer hover:text-sky-5 hover:bg-gray-2/70 rounded py-1 my-1 px-2"
              @click="store.currentNavName = 'search'"
              :class="{ 'text-sky-5': store.currentNavName === 'search' }">
              <PhMagnifyingGlass :weight="store.currentNavName === 'search' ? 'fill' : 'regular'" size="28" />
            </div>
          </template>
        </n-tooltip>

        <n-tooltip placement="right" trigger="hover" :show-arrow="false">
          收藏
          <template #trigger>
            <div
              class="flex cursor-pointer hover:text-sky-5 hover:bg-gray-2/70 rounded py-1 my-1 px-2"
              @click="store.currentNavName = 'fav'"
              :class="{ 'text-sky-5': store.currentNavName === 'fav' }">
              <PhStar :weight="store.currentNavName === 'fav' ? 'fill' : 'regular'" size="28" />
            </div>
          </template>
        </n-tooltip>

        <n-tooltip placement="right" trigger="hover" :show-arrow="false">
          稍后再看
          <template #trigger>
            <div
              class="flex cursor-pointer hover:text-sky-5 hover:bg-gray-2/70 rounded py-1 my-1 px-2"
              @click="store.currentNavName = 'watch_later'"
              :class="{ 'text-sky-5': store.currentNavName === 'watch_later' }">
              <PhClock :weight="store.currentNavName === 'watch_later' ? 'fill' : 'regular'" size="28" />
            </div>
          </template>
        </n-tooltip>

        <n-tooltip placement="right" trigger="hover" :show-arrow="false">
          追番追剧
          <template #trigger>
            <div
              class="flex cursor-pointer hover:text-sky-5 hover:bg-gray-2/70 rounded py-1 my-1 px-2"
              @click="store.currentNavName = 'bangumi_follow'"
              :class="{ 'text-sky-5': store.currentNavName === 'bangumi_follow' }">
              <PhHeart :weight="store.currentNavName === 'bangumi_follow' ? 'fill' : 'regular'" size="28" />
            </div>
          </template>
        </n-tooltip>

        <n-tooltip placement="right" trigger="hover" :show-arrow="false">
          下载
          <template #trigger>
            <n-badge :value="store.uncompletedProgressesCount" :offset="[-7, 7]">
              <div
                ref="downloadButtonRef"
                class="flex cursor-pointer hover:text-sky-5 hover:bg-gray-2/70 rounded py-1 my-1 px-2"
                @click="store.currentNavName = 'download'"
                :class="{ 'text-sky-5': store.currentNavName === 'download' }">
                <PhDownload :weight="store.currentNavName === 'download' ? 'fill' : 'regular'" size="28" />
              </div>
            </n-badge>
          </template>
        </n-tooltip>

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
      <div class="relative w-full overflow-hidden">
        <transition name="fade">
          <SearchPane class="absolute inset-0" v-show="store.currentNavName === 'search'" ref="searchPaneRef" />
        </transition>
        <transition name="fade">
          <FavPane class="absolute inset-0" v-show="store.currentNavName === 'fav'" />
        </transition>
        <transition name="fade">
          <WatchLaterPane class="absolute inset-0" v-show="store.currentNavName === 'watch_later'" />
        </transition>
        <transition name="fade">
          <BangumiFollowPane class="absolute inset-0" v-show="store.currentNavName === 'bangumi_follow'" />
        </transition>
        <transition name="fade">
          <DownloadPane class="absolute inset-0" v-show="store.currentNavName === 'download'" />
        </transition>
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

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
