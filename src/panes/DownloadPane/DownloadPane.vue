<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { commands, DownloadProgress, DownloadTaskState, events } from '../../bindings.ts'
import { useStore } from '../../store.ts'
import UncompletedProgresses from './components/UncompletedProgresses.vue'
import CompletedProgresses from './components/CompletedProgresses.vue'
import DownloadDirInput from './components/DownloadDirInput.vue'
import { PhCheckCircle, PhCloudArrowDown } from '@phosphor-icons/vue'

export type ProgressData = DownloadProgress & {
  state: DownloadTaskState
  percentage: number
  stateIndicator: string
  taskIndicator: string
}

const store = useStore()

const currentTabName = ref<'uncompleted' | 'completed'>('uncompleted')

const uncompletedProgressesRef = ref<InstanceType<typeof UncompletedProgresses>>()
const completedProgressesRef = ref<InstanceType<typeof CompletedProgresses>>()

onMounted(async () => {
  await events.downloadEvent.listen(({ payload: { event, data } }) => {
    if (event === 'Speed') {
      store.downloadSpeed = data.speed
    } else if (event === 'TaskCreate') {
      const { progress, state } = data
      const taskId = progress.task_id
      const progressData: ProgressData = {
        ...progress,
        state,
        percentage: 0,
        stateIndicator: '',
        taskIndicator: '',
      }
      store.updateProgresses((progresses) => {
        progresses.set(taskId, progressData)
      })
    } else if (event === 'TaskStateUpdate') {
      const { task_id, state } = data

      store.updateProgresses((progresses) => {
        const progressData = progresses.get(task_id)
        if (progressData === undefined) {
          return
        }

        let stateIndicator = ''
        if (state === 'Pending') {
          stateIndicator = '排队中'
        } else if (state === 'Downloading') {
          stateIndicator = '下载中'
        } else if (state === 'Paused') {
          stateIndicator = '已暂停'
        } else if (state === 'Completed') {
          stateIndicator = '下载完成'
        } else if (state === 'Failed') {
          stateIndicator = '下载失败'
        }

        progressData.state = state
        progressData.stateIndicator = stateIndicator
      })
    } else if (event === 'TaskSleeping') {
      const { task_id, remaining_sec } = data
      store.updateProgresses((progresses) => {
        const progressData = progresses.get(task_id)
        if (progressData !== undefined) {
          progressData.taskIndicator = `将在${remaining_sec}秒后继续下载`
        }
      })
    } else if (event === 'TaskDelete') {
      const { task_id } = data
      store.updateProgresses((progresses) => {
        progresses.delete(task_id)
      })
    } else if (event === 'ProgressPreparing') {
      const { task_id } = data
      store.updateProgresses((progresses) => {
        const progressData = progresses.get(task_id)
        if (progressData !== undefined) {
          progressData.taskIndicator = '正在准备下载'
        }
      })
    } else if (event === 'ProgressUpdate') {
      const progress = data.progress
      store.updateProgresses((progresses) => {
        const progressData = progresses.get(progress.task_id)
        if (progressData === undefined) {
          return
        }

        Object.assign(progressData, progress)

        const videoTask = progressData.video_task
        const audioTask = progressData.audio_task
        const mergeTask = progressData.merge_task
        const embedChapterTask = progressData.embed_chapter_task
        const embedSkipTask = progressData.embed_skip_task
        const danmakuTask = progressData.danmaku_task
        const subtitleTask = progressData.subtitle_task
        const coverTask = progressData.cover_task
        const nfoTask = progressData.nfo_task
        const jsonTask = progressData.json_task

        const danmakuSelected = danmakuTask.xml_selected || danmakuTask.ass_selected || danmakuTask.json_selected
        const embedding =
          (embedChapterTask.selected && !embedChapterTask.completed) ||
          (embedSkipTask.selected && !embedSkipTask.completed)

        if (videoTask.selected && !videoTask.completed && videoTask.content_length > 0) {
          const chunkCount = progressData.video_task.chunks.length
          const completedChunks = progressData.video_task.chunks.filter((chunk) => chunk.completed).length
          progressData.percentage = (completedChunks / chunkCount) * 100
          progressData.taskIndicator = `视频分片 ${completedChunks}/${chunkCount}`
        } else if (audioTask.selected && !audioTask.completed && audioTask.content_length > 0) {
          const chunkCount = progressData.audio_task.chunks.length
          const completedChunks = progressData.audio_task.chunks.filter((chunk) => chunk.completed).length
          progressData.percentage = (completedChunks / chunkCount) * 100
          progressData.taskIndicator = `音频分片 ${completedChunks}/${chunkCount}`
        } else if (mergeTask.selected && !mergeTask.completed && embedding) {
          progressData.percentage = 100
          progressData.taskIndicator = '自动合并+嵌入章节元数据'
        } else if (mergeTask.selected && !mergeTask.completed) {
          progressData.percentage = 100
          progressData.taskIndicator = '自动合并'
        } else if (embedding) {
          progressData.percentage = 100
          progressData.taskIndicator = '嵌入章节元数据'
        } else if (danmakuSelected && !danmakuTask.completed) {
          progressData.percentage = 100
          progressData.taskIndicator = '弹幕'
        } else if (subtitleTask.selected && !subtitleTask.completed) {
          progressData.percentage = 100
          progressData.taskIndicator = '字幕'
        } else if (coverTask.selected && !coverTask.completed) {
          progressData.percentage = 100
          progressData.taskIndicator = '封面'
        } else if (nfoTask.selected && !nfoTask.completed) {
          progressData.percentage = 100
          progressData.taskIndicator = 'nfo刮削'
        } else if (jsonTask.selected && !jsonTask.completed) {
          progressData.percentage = 100
          progressData.taskIndicator = 'json刮削'
        }
      })
    }
  })

  const result = await commands.restoreDownloadTasks()
  if (result.status === 'error') {
    console.error(result.error)
  }
})
</script>

<template>
  <div class="h-full flex flex-col overflow-auto" v-if="store.config !== undefined">
    <div class="flex items-center m-2 mb-0">
      <DownloadDirInput />
      <span class="ml-2 whitespace-nowrap">下载速度：{{ store.downloadSpeed }}</span>
    </div>
    <n-tabs
      class="h-full overflow-auto"
      type="line"
      v-model:value="currentTabName"
      animated
      size="small"
      placement="bottom">
      <n-tab-pane class="h-full p-0! overflow-auto flex flex-col" name="uncompleted">
        <template #tab>
          <n-badge :value="store.uncompletedProgressesCount" :offset="[5, -3]" class="tab-badge-wrapper">
            <PhCloudArrowDown :weight="currentTabName === 'uncompleted' ? 'fill' : 'regular'" size="20" />
            <span class="ml-1">下载中</span>
          </n-badge>
        </template>
        <UncompletedProgresses ref="uncompletedProgressesRef" />
      </n-tab-pane>

      <n-tab-pane class="h-full p-0! overflow-auto flex flex-col" name="completed">
        <template #tab>
          <PhCheckCircle :weight="currentTabName === 'completed' ? 'fill' : 'regular'" size="20" />
          <span class="ml-1">已完成</span>
        </template>

        <CompletedProgresses ref="completedProgressesRef" />
      </n-tab-pane>

      <template #suffix>
        <n-pagination
          v-if="currentTabName === 'uncompleted' && uncompletedProgressesRef"
          class="ml-auto mr-2"
          :page-count="uncompletedProgressesRef.pageCount"
          v-model:page="uncompletedProgressesRef.currentPage" />

        <n-pagination
          v-else-if="currentTabName === 'completed' && completedProgressesRef"
          class="ml-auto mr-2"
          :page-count="completedProgressesRef.pageCount"
          v-model:page="completedProgressesRef.currentPage" />
      </template>
    </n-tabs>
  </div>
</template>
<style scoped>
:deep(.n-tabs-nav-scroll-wrapper) {
  @apply mt-2 h-9;
}

:deep(.n-tabs-nav__suffix) {
  @apply important-border-0;
}

:deep(.n-tabs-nav-scroll-wrapper) {
  @apply overflow-visible;
}

:deep(.v-x-scroll) {
  @apply overflow-visible;
}
</style>
