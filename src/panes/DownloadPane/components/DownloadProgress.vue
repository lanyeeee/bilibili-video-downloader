<script setup lang="tsx">
import { path } from '@tauri-apps/api'
import { commands, DownloadTaskState } from '../../../bindings.ts'
import { useStore } from '../../../store.ts'
import {
  PhWarningCircle,
  PhCloudArrowDown,
  PhPause,
  PhClock,
  PhGoogleChromeLogo,
  PhFolderOpen,
  PhFileVideo,
  PhMagnifyingGlass,
} from '@phosphor-icons/vue'
import { ProgressProps } from 'naive-ui'
import UpInfoBadge from '../../../components/UpInfoBadge.vue'
import { computed, DeepReadonly, inject } from 'vue'
import ColorfulTag from '../../../components/ColorfulTag.vue'
import { searchPaneRefKey } from '../../../injection_keys.ts'
import { ProgressData } from '../DownloadPane.vue'
import { ensureHttps } from '../../../utils.tsx'
import IconButton from '../../../components/IconButton.vue'

const store = useStore()

const searchPaneRef = inject(searchPaneRefKey)

const props = defineProps<{
  p: DeepReadonly<ProgressData>
}>()

const selectedIds = defineModel<Set<string>>('selectedIds', { required: true })

function handleProgressContextMenu() {
  const taskId = props.p.task_id
  if (selectedIds.value.has(taskId)) {
    return
  }
  selectedIds.value.clear()
  selectedIds.value.add(taskId)
}

async function handleProgressDoubleClick() {
  const state = props.p.state
  const taskId = props.p.task_id

  if (state === 'Downloading' || state === 'Pending') {
    await commands.pauseDownloadTasks([taskId])
  } else if (state === 'Failed' || state === 'Paused') {
    await commands.resumeDownloadTasks([taskId])
  }
}

function stateToStatus(state: DownloadTaskState): ProgressProps['status'] {
  if (state === 'Completed') {
    return 'success'
  } else if (state === 'Paused') {
    return 'warning'
  } else if (state === 'Failed') {
    return 'error'
  } else {
    return 'default'
  }
}

function stateToColorClass(state: DownloadTaskState) {
  if (state === 'Downloading') {
    return 'text-blue-500'
  } else if (state === 'Pending') {
    return 'text-gray-500'
  } else if (state === 'Paused') {
    return 'text-yellow-500'
  } else if (state === 'Failed') {
    return 'text-red-500'
  } else if (state === 'Completed') {
    return 'text-green-500'
  }

  return ''
}

async function showMp4InFileManager(episodeDir: string, filename: string) {
  if (store.config === undefined) {
    return
  }

  let mp4filename = `${filename}.mp4`
  const mp4Path = await path.join(episodeDir, mp4filename)

  const result = await commands.showPathInFileManager(mp4Path)
  if (result.status === 'error') {
    console.error(result.error)
  }
}

async function showEpisodeDirInFileManager(episodeDir: string) {
  if (store.config === undefined) {
    return
  }

  const result = await commands.showPathInFileManager(episodeDir)
  if (result.status === 'error') {
    console.error(result.error)
  }
}

const href = computed<string>(() => {
  if (props.p.episode_type === 'Normal') {
    let href = `https://www.bilibili.com/video/${props.p.bvid}/`
    if (props.p.part_order !== null) {
      href += `?p=${props.p.part_order}`
    }
    return href
  } else if (props.p.episode_type === 'Bangumi') {
    return `https://www.bilibili.com/bangumi/play/ep${props.p.ep_id}`
  } else if (props.p.episode_type === 'Cheese') {
    return `https://www.bilibili.com/cheese/play/ep${props.p.ep_id}`
  }
  return 'https://www.bilibili.com/'
})

function handleSearchClick() {
  if (props.p.episode_type === 'Normal' && props.p.bvid !== null) {
    searchPaneRef?.value?.search(props.p.bvid, 'Normal')
  } else if (props.p.episode_type === 'Bangumi' && props.p.ep_id !== null) {
    searchPaneRef?.value?.search(`ep${props.p.ep_id}`, 'Bangumi')
  } else if (props.p.episode_type === 'Cheese' && props.p.ep_id !== null) {
    searchPaneRef?.value?.search(`ep${props.p.ep_id}`, 'Cheese')
  }
}
</script>

<template>
  <div
    class="p-2 mb-2 rounded-lg flex flex-col border border-solid border-gray-2"
    @contextmenu="handleProgressContextMenu"
    @dblclick="handleProgressDoubleClick">
    <div class="flex">
      <img
        class="w-224px h-140px rounded-lg object-cover lazyload"
        :data-src="`${ensureHttps(p.cover_task.url)}@672w_378h_1c.webp`"
        :key="p.cover_task.url"
        alt=""
        draggable="false" />

      <div class="ml-2 flex flex-col w-full overflow-hidden">
        <div class="text-lg font-bold line-clamp-2" :title="p.episode_title">{{ p.episode_title }}</div>

        <ColorfulTag
          color="violet"
          class="bg-violet-2 w-fit font-bold line-clamp-1"
          v-if="p.part_title !== null"
          :title="p.part_title">
          P{{ p.part_order }} {{ p.part_title }}
        </ColorfulTag>

        <div class="mt-auto flex gap-1 flex-wrap pt-2" title="任务内容">
          <ColorfulTag v-if="p.video_task.selected" color="blue">
            视频(编码:{{ p.video_task.codec_type }} 画质:{{ p.video_task.video_quality }})
          </ColorfulTag>
          <ColorfulTag v-if="p.audio_task.selected" color="blue">
            音频(音质:{{ p.audio_task.audio_quality }})
          </ColorfulTag>

          <ColorfulTag v-if="p.video_process_task.merge_selected" color="purple">自动合并</ColorfulTag>
          <ColorfulTag v-if="p.video_process_task.embed_chapter_selected" color="purple">标记章节</ColorfulTag>
          <ColorfulTag v-if="p.video_process_task.embed_skip_selected" color="purple">标记广告</ColorfulTag>

          <ColorfulTag v-if="p.danmaku_task.xml_selected" color="green">xml弹幕</ColorfulTag>
          <ColorfulTag v-if="p.danmaku_task.ass_selected" color="green">ass弹幕</ColorfulTag>
          <ColorfulTag v-if="p.danmaku_task.json_selected" color="green">json弹幕</ColorfulTag>

          <ColorfulTag v-if="p.subtitle_task.selected" color="amber">字幕</ColorfulTag>
          <ColorfulTag v-if="p.cover_task.selected" color="amber">封面</ColorfulTag>

          <ColorfulTag v-if="p.nfo_task.selected" color="rose">nfo刮削</ColorfulTag>
          <ColorfulTag v-if="p.json_task.selected" color="rose">json刮削</ColorfulTag>
        </div>
      </div>
    </div>
    <div class="flex mt-2 gap-2">
      <UpInfoBadge
        class="w-40"
        v-if="p.up_name !== null && p.up_avatar !== null && p.up_uid !== null"
        :up-name="p.up_name"
        :up-avatar="p.up_avatar"
        :up-uid="p.up_uid" />

      <div v-if="p.state !== 'Completed'" :class="[stateToColorClass(p.state), 'flex items-center w-100']">
        <n-icon :size="22">
          <PhCloudArrowDown v-if="p.state === 'Downloading'" />
          <PhClock v-else-if="p.state === 'Pending'" />
          <PhPause v-else-if="p.state === 'Paused'" />
          <PhWarningCircle v-else-if="p.state === 'Failed'" />
        </n-icon>
        <span class="whitespace-nowrap mr-2">{{ p.stateIndicator }}</span>
        <n-progress
          v-if="p.taskIndicator !== ''"
          :status="stateToStatus(p.state)"
          :percentage="p.percentage"
          :processing="p.state === 'Downloading'">
          {{ p.taskIndicator }}
        </n-progress>
      </div>
      <div v-else-if="p.completed_ts !== null" title="完成时间" class="flex items-center">
        <n-time class="font-bold" unix :time="p.completed_ts" />
      </div>

      <div class="ml-auto flex gap-2 items-center">
        <IconButton
          v-if="p.state === 'Completed' && p.video_task.selected"
          title="打开mp4目录"
          @click="showMp4InFileManager(p.episode_dir, p.filename)">
          <PhFileVideo :size="24" />
        </IconButton>
        <IconButton
          v-if="p.state === 'Completed'"
          title="打开下载目录"
          @click="showEpisodeDirInFileManager(p.episode_dir)">
          <PhFolderOpen :size="24" />
        </IconButton>
        <IconButton title="在下载器内搜索" @click="handleSearchClick">
          <PhMagnifyingGlass :size="24" />
        </IconButton>
        <IconButton title="在浏览器中打开" :href="href">
          <PhGoogleChromeLogo :size="24" />
        </IconButton>
      </div>
    </div>
  </div>
</template>
